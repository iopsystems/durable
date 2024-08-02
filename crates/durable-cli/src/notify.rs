use anyhow::Context;
use durable_client::{DurableClient, Task};
use serde_json::value::RawValue;

use crate::CommonOptions;

#[derive(Debug, clap::Args)]

pub(crate) struct Notify {
    /// The task that we want to notify.
    task: i64,

    /// The event that we are notifying the task with.
    event: String,

    /// JSON data to pass to the task.
    data: Option<String>,
}

impl Notify {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;
        let client = DurableClient::new(pool.clone())?;
        let task = Task::from_id(self.task);

        let data = self.data.as_deref().unwrap_or("null");
        let data: &RawValue =
            serde_json::from_str(data).context("provided event data was not valid json")?;

        task.notify(&self.event, data, &client).await?;

        Ok(())
    }
}
