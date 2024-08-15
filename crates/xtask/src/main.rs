use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::prelude::*;

mod dev;
mod gen;
mod migrate;

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Generate(self::gen::Gen),
    Migrate(self::migrate::Migrate),
    Dev(self::dev::Dev),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_writer(std::io::stdout),
        )
        .init();

    match args.command {
        Command::Generate(cmd) => cmd.run(),
        Command::Migrate(cmd) => cmd.run(),
        Command::Dev(cmd) => cmd.run(),
    }
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Metadata {
        workspace_root: PathBuf,
    }

    let sh = xshell::Shell::new()?;
    let metadata = xshell::cmd!(sh, "cargo metadata --format-version 1")
        .quiet()
        .read()?;

    let metadata: Metadata = serde_json::from_str(&metadata)
        .context("failed to deserialize the output of cargo metadata")?;

    Ok(metadata.workspace_root)
}
