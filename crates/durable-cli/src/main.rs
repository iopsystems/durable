//!

use std::str::FromStr;

use anyhow::Context;
use clap::Parser;
use sqlx::ConnectOptions;
use tokio::sync::OnceCell;
use tracing_subscriber::prelude::*;

mod launch;
mod logs;

#[derive(Debug, clap::Parser)]
struct Args {
    #[command(flatten)]
    common: CommonOptions,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, clap::Subcommand)]
enum Commands {
    Launch(self::launch::Launch),
    Logs(self::logs::Logs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    match args.command {
        Commands::Launch(cmd) => cmd.run(&args.common).await,
        Commands::Logs(cmd) => cmd.run(&args.common).await,
    }
}

#[derive(Debug, clap::Args)]
struct CommonOptions {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,

    #[arg(skip)]
    pool: OnceCell<sqlx::PgPool>,
}

impl CommonOptions {
    pub async fn pool(&self) -> anyhow::Result<sqlx::PgPool> {
        self.pool
            .get_or_try_init(|| async {
                let options = sqlx::postgres::PgConnectOptions::from_str(&self.database_url)
                    .context("failed to parse database URL")?
                    .log_statements(log::LevelFilter::Debug);

                sqlx::PgPool::connect_with(options)
                    .await
                    .context("failed to connect to the database")
            })
            .await
            .map(|pool| pool.clone())
    }
}
