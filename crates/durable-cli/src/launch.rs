use std::path::PathBuf;

use anyhow::Context;
use durable_client::{DurableClient, ProgramOptions};
use futures_util::TryStreamExt;
use serde_json::value::RawValue;

use crate::CommonOptions;

#[derive(Debug, clap::Args)]
pub(crate) struct Launch {
    /// A name to assign to this task.
    name: String,

    /// The wasm binary to launch as a task.
    wasm: PathBuf,

    #[arg(long)]
    data: Option<String>,

    /// Wait for the workflow to complete and print logs as we go.
    #[arg(long, short = 'f')]
    tail: bool
}

impl Launch {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;
        let client = DurableClient::new(pool.clone())?;

        if self.name.is_empty() {
            anyhow::bail!("the task name must not be an empty string");
        }

        let data = self.data.as_deref().unwrap_or("null");
        let data: &RawValue =
            serde_json::from_str(data).context("provided task data was not valid json")?;

        let options = ProgramOptions::from_file(&self.wasm)
            .with_context(|| format!("failed to read `{}`", self.wasm.display()))?;
        let program = client.program(options).await?;
        let task = client.launch(&self.name, &program, data).await?;

        println!("launched new task with id {}", task.id());

        if self.tail {
            let mut logs = std::pin::pin!(task.follow_logs(&client));

            while let Some(message) = logs.try_next().await? {
                print!("{message}")
            }
        }

        Ok(())
    }
}
