use std::path::{Path, PathBuf};

use anyhow::Context;
use durable_client::{DurableClient, Program, ProgramOptions};

mod basic;
mod sqlx;

async fn load_binary(client: &DurableClient, name: &str) -> anyhow::Result<Program> {
    let program = client
        .program(
            ProgramOptions::from_file(crate::test_binary(name))
                .with_context(|| format!("failed to load {name} file"))?,
        )
        .await?;

    Ok(program)
}

fn test_binary(name: impl AsRef<Path>) -> PathBuf {
    let Some(bindir) = std::env::var_os("DURABLE_TEST_BIN_DIR") else {
        panic!(
            "DURABLE_TEST_BIN_DIR env var is not set. Are you running tests without using `cargo \
             nextest run`?"
        );
    };

    let name = name.as_ref();
    let mut path = PathBuf::from(bindir);
    path.push(name);

    if !path.exists() {
        panic!(
            "Attempted to request non-existant test binary `{}`",
            name.display()
        );
    }

    path
}
