use futures_util::TryStreamExt;
use serde::Deserialize;
use sqlx::types::Json;

use crate::CommonOptions;

#[derive(Debug, clap::Parser)]
pub(crate) struct Logs {
    /// The id of the task we want to see the logs for.
    pub task: i64,
}

#[derive(Debug, Deserialize)]
struct Event {
    data: String,
}

impl Logs {
    pub async fn run(self, options: &CommonOptions) -> anyhow::Result<()> {
        let pool = options.pool().await?;

        let exists = sqlx::query!("SELECT id FROM task WHERE id = $1", self.task)
            .fetch_optional(&pool)
            .await?
            .is_some();

        if !exists {
            anyhow::bail!("there is no task with id {}", self.task);
        }

        let mut stream = sqlx::query!(
            r#"
            SELECT
                value as "data!: Json<Event>"
            FROM event
            WHERE task_id = $1
              AND label IN ('durable::print')
            ORDER BY index ASC
            "#,
            self.task
        )
        .fetch(&pool);

        while let Some(record) = stream.try_next().await? {
            let event = record.data.0;
            print!("{}", event.data);
        }

        Ok(())
    }
}
