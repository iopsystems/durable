use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tracing_subscriber::prelude::*;

mod claude;
mod dev;
mod gen;
mod migrate;
mod package;
mod publish;

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Claude(self::claude::Claude),
    Generate(self::gen::Gen),
    Migrate(self::migrate::Migrate),
    Dev(self::dev::Dev),
    Package(self::package::Package),
    Publish(self::publish::Publish),
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
        Command::Claude(cmd) => cmd.run(),
        Command::Generate(cmd) => cmd.run(),
        Command::Migrate(cmd) => cmd.run(),
        Command::Dev(cmd) => cmd.run(),
        Command::Package(cmd) => cmd.run(),
        Command::Publish(cmd) => cmd.run(),
    }
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .context("failed to run `cargo metadata`")?;

    Ok(metadata.workspace_root.into_std_path_buf())
}
