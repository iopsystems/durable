//!

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::prelude::*;

mod launch;

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
    }
}

#[derive(Debug, clap::Args)]
struct CommonOptions {
    #[arg(long, env = "DATABASE_URL")]
    database_url: String,
}

impl CommonOptions {
    pub async fn pool(&self) -> anyhow::Result<sqlx::PgPool> {
        sqlx::PgPool::connect(&self.database_url)
            .await
            .context("failed to connect to the database")
    }
}
