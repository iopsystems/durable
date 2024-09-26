use std::time::Duration;

use durable_runtime::{Config, WorkerBuilder, WorkerHandle};
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

    Ok(WorkerShutdownGuard {
        handle,
        _task: Some(task),
    })
}

pub struct WorkerShutdownGuard {
    handle: WorkerHandle,
    _task: Option<JoinHandle<anyhow::Result<()>>>,
}

impl Drop for WorkerShutdownGuard {
    fn drop(&mut self) {
        self.handle.shutdown();

        // if std::thread::panicking() {
        //     return;
        // }

        // tokio::task::block_in_place(|| {
        //     if let Some(task) = self.task.take() {
        //         let future = tokio::time::timeout(Duration::from_secs(1),
        // task);         match Handle::current().block_on(future) {
        //             Ok(Ok(Ok(()))) => (),
        //             Ok(Ok(Err(e))) => {
        //                 eprintln!("durable runtime exited with an error:
        // {e:?}");                 panic!("durable runtime exited with
        // an error")             }
        //             Ok(Err(e)) => std::panic::resume_unwind(e.into_panic()),
        //             Err(_) => {
        //                 eprintln!("warning: failed to wait for the runtime
        // task to complete")             }
        //         }
        //     }
        // })
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
