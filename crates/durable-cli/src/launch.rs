use std::path::PathBuf;

use anyhow::Context;
use serde_json::value::RawValue;
use sqlx::types::Json;

use crate::CommonOptions;

#[derive(Debug, clap::Args)]
pub(crate) struct Launch {
    /// A name to assign to this task.
    name: String,

    /// The wasm binary to launch as a task.
    wasm: PathBuf,

    #[arg(long)]
    data: Option<String>,
}

impl Launch {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;

        if self.name.is_empty() {
            anyhow::bail!("the task name must not be an empty string");
        }

        let wasm = std::fs::read(&self.wasm)
            .with_context(|| format!("failed to read `{}`", self.wasm.display()))?;

        let data = self.data.as_deref().unwrap_or("null");
        let data: &RawValue =
            serde_json::from_str(data).context("provided task data was not valid json")?;

        let id = sqlx::query!(
            "
            INSERT INTO task(name, wasm, data, running_on)
            VALUES
                ($1, $2, $3, NULL)
            RETURNING id
            ",
            &self.name,
            &wasm,
            Json(data) as Json<&RawValue>
        )
        .fetch_one(&pool)
        .await?
        .id;

        println!("launched new task with id {id}");

        Ok(())
    }
}
