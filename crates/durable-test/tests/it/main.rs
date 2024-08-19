use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use durable_client::{DurableClient, Program, ProgramOptions, Task};
use durable_runtime::Config;
use futures::TryStreamExt;

mod basic;

pub fn test_config() -> Config {
    Config::default()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(1))
}

/// Launch a task that tails the logs of the provided task.
pub fn tail_logs(client: &DurableClient, task: &Task) {
    let task = task.clone();
    let client = client.clone();

    tokio::task::spawn(async move {
        let future = async {
            let mut stream = std::pin::pin!(task.follow_logs(&client));

            while let Some(logs) = stream.try_next().await? {
                print!("{logs}");
            }

            anyhow::Ok(())
        };

        if let Err(e) = future.await {
            eprintln!("failed to tail the task logs: {e}")
        }
    });
}

pub async fn load_binary(client: &DurableClient, name: &str) -> anyhow::Result<Program> {
    let program = client
        .program(
            ProgramOptions::from_file(crate::test_binary(name))
                .with_context(|| format!("failed to load {name} file"))?,
        )
        .await?;

    Ok(program)
}

pub fn test_binary(name: impl AsRef<Path>) -> PathBuf {
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
