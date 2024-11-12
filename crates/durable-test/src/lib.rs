use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use durable_runtime::{Config, WorkerBuilder, WorkerHandle};
use futures::FutureExt;
use tokio::task::JoinHandle;

pub async fn spawn_worker(pool: sqlx::PgPool) -> anyhow::Result<WorkerShutdownGuard> {
    spawn_worker_with(
        pool,
        Config::new()
            .suspend_margin(Duration::from_secs(1))
            .suspend_timeout(Duration::from_secs(1)),
    )
    .await
}

pub async fn spawn_worker_with(
    pool: sqlx::PgPool,
    config: Config,
) -> anyhow::Result<WorkerShutdownGuard> {
    let mut wasmconfig = wasmtime::Config::new();
    wasmconfig
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .debug_info(true)
        .cache_config_load_default()?;
    let mut worker = WorkerBuilder::new(pool)
        .config(config.debug_emit_task_logs(true))
        .wasmtime_config(wasmconfig)
        .validate_database(false)
        .build()
        .await?;

    let handle = worker.handle();
    let task = tokio::spawn(async move { worker.run().await });

    Ok(WorkerShutdownGuard { handle, task })
}

pub struct WorkerShutdownGuard {
    handle: WorkerHandle,
    task: JoinHandle<anyhow::Result<()>>,
}

impl WorkerShutdownGuard {
    pub fn handle(&self) -> WorkerHandle {
        self.handle.clone()
    }
}

impl Future for WorkerShutdownGuard {
    type Output = anyhow::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let result = std::task::ready!(self.task.poll_unpin(cx));

        Poll::Ready(match result {
            Ok(result) => result,
            Err(e) => match e.try_into_panic() {
                Ok(payload) => std::panic::resume_unwind(payload),
                Err(e) => Err(anyhow::anyhow!(e)),
            },
        })
    }
}

impl Drop for WorkerShutdownGuard {
    fn drop(&mut self) {
        self.handle.shutdown();
    }
}

#[ctor::ctor]
fn setup_tracing() {
    use tracing_subscriber::prelude::*;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}
