use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;

mod generate;

#[derive(Debug, clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Generate(self::generate::Generate),
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Generate(cmd) => cmd.run(),
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
