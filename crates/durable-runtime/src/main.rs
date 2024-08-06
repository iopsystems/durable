use anyhow::Context;
use clap::Parser;
use durable_runtime::{WorkerBuilder, WorkerHandle};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(unix)]
    unsafe { backtrace_on_stack_overflow::enable() };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let args = Args::parse();
    let pool = sqlx::PgPool::connect(&args.database_url)
        .await
        .context("failed to connect to the database")?;

    let mut config = wasmtime::Config::new();
    config.cache_config_load_default()?;
    config.async_support(true);
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    config.wasm_threads(false);

    let engine = wasmtime::Engine::new(&config)?;

    let mut worker = WorkerBuilder::new(pool).engine(engine).build().await?;

    let handle = worker.handle();

    let signal = tokio::task::spawn(async move {
        struct DropGuard(WorkerHandle);

        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.shutdown();
            }
        }

        let _handle = DropGuard(handle);

        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};

            let mut sigint = signal(SignalKind::interrupt())?;
            let mut sigterm = signal(SignalKind::terminate())?;

            tokio::select! {
                _ = sigint.recv() => (),
                _ = sigterm.recv() => (),
            }
        }

        #[cfg(not(unix))]
        tokio::signal::ctrl_c().await?;

        tracing::info!("Got signal. Shutting down!");

        anyhow::Ok(())
    });

    tracing::info!("durable-server starting up!");
    worker.run().await?;

    signal
        .await
        .context("signal task exited early with an error")?
        .context("signal task exited early with an error")?;

    Ok(())
}
