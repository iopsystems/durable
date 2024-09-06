use durable_client::{DurableClient, Task};
use futures_util::stream::BoxStream;
use futures_util::TryStreamExt;

use crate::CommonOptions;

/// Print the log messages emitted by a durable task.
#[derive(Debug, clap::Parser)]
pub(crate) struct Logs {
    /// The id of the task we want to see the logs for.
    pub task: i64,

    /// Wait for the workflow to complete and print logs as we go.
    #[arg(long, short = 'f')]
    pub tail: bool,
}

impl Logs {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;
        let client = DurableClient::new(pool)?;
        let task = Task::from_id(self.task);

        let mut stream: BoxStream<_> = if self.tail {
            Box::pin(task.follow_logs(&client))
        } else {
            Box::pin(task.read_logs(&client))
        };

        while let Some(message) = stream.try_next().await? {
            print!("{message}");
        }

        Ok(())
    }
}
