use std::time::Duration;

use durable_runtime::{Config, WorkerBuilder, WorkerHandle};
use tokio::task::JoinHandle;

pub async fn spawn_worker(pool: sqlx::PgPool) -> anyhow::Result<WorkerShutdownGuard> {
    let mut config = wasmtime::Config::new();
    config
        .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .debug_info(true)
        .cache_config_load_default()?;
    let mut worker = WorkerBuilder::new(pool)
        .config(
            Config::new()
                .suspend_margin(Duration::from_secs(1))
                .suspend_timeout(Duration::from_secs(1)),
        )
        .wasmtime_config(config)
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
