use durable_client::{DurableClient, Task};
use futures_util::TryStreamExt;

use crate::CommonOptions;

#[derive(Debug, clap::Parser)]
pub(crate) struct Logs {
    /// The id of the task we want to see the logs for.
    pub task: i64,
}

impl Logs {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;
        let client = DurableClient::new(pool)?;
        let task = Task::from_id(self.task);

        let mut stream = std::pin::pin!(task.read_logs(&client));

        while let Some(message) = stream.try_next().await? {
            print!("{message}");
        }

        Ok(())
    }
}
