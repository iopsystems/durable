use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use durable_runtime::{Config, WorkerBuilder, WorkerHandle};
use futures::FutureExt;
use tokio::task::JoinHandle;
use wasmtime::{Cache, CacheConfig};

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
    let builder = make_worker_builder(pool, config);
    spawn_from_builder(builder).await
}

/// Spawn a worker with DST hooks injected.
///
/// Returns the guard for the worker. The caller retains access to the
/// scheduler, clock, and entropy for test-side inspection and control.
pub async fn spawn_worker_with_dst(
    pool: sqlx::PgPool,
    config: Config,
    scheduler: Arc<dyn durable_runtime::Scheduler>,
    clock: Arc<dyn durable_runtime::Clock>,
    entropy: Arc<dyn durable_runtime::Entropy>,
) -> anyhow::Result<WorkerShutdownGuard> {
    let builder = make_worker_builder(pool, config)
        .scheduler(scheduler)
        .clock(clock)
        .entropy(entropy);
    spawn_from_builder(builder).await
}

fn make_worker_builder(pool: sqlx::PgPool, config: Config) -> WorkerBuilder {
    let mut wasmconfig = wasmtime::Config::new();
    wasmconfig
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .debug_info(true)
        .cache(CacheConfig::from_file(None).and_then(Cache::new).ok());
    WorkerBuilder::new(pool)
        .config(config.debug_emit_task_logs(true))
        .wasmtime_config(wasmconfig)
        .validate_database(false)
}

async fn spawn_from_builder(builder: WorkerBuilder) -> anyhow::Result<WorkerShutdownGuard> {
    let mut worker = builder.build().await?;

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
