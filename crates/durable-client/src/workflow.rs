use async_stream::try_stream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use serde_json::Value;
use sqlx::types::Json;

use crate::error::ErrorImpl;
use crate::{DurableClient, DurableError};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub index: i32,
    pub label: String,
    pub value: Value,
}

/// A handle for a workflow task.
///
/// Generally, you should get this by calling [`DurableClient::launch`] but you
/// can also construct it directly from an id.
#[derive(Clone, Debug)]
pub struct Task {
    pub(crate) id: i64,
}

impl Task {
    /// Create a new `Task` directly from an id.
    ///
    /// This does no validation on the id. As such, calling anything that
    /// attempts to fetch the corresponding task from the database is liable to
    /// result in an error.
    pub fn from_id(id: i64) -> Self {
        Self { id }
    }

    /// Get the id of this task.
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn events<'a>(
        &'a self,
        client: &DurableClient,
    ) -> impl Stream<Item = Result<Event, DurableError>> + '_ {
        let pool = client.pool.clone();

        try_stream! {
            let mut conn = pool.acquire().await?;
            let mut count = 0;

            let mut events = sqlx::query!(
                r#"
                SELECT
                    index,
                    label,
                    value as "value!: Json<Value>"
                FROM durable.event
                WHERE task_id = $1
                ORDER BY index ASC
                "#,
                self.id
            )
            .fetch(&mut *conn);

            while let Some(record) = events.try_next().await? {
                count += 1;

                yield Event {
                    index: record.index,
                    label: record.label,
                    value: record.value.0
                }
            }

            drop(events);

            if count == 0 {
                let exists = sqlx::query!("SELECT id FROM durable.task WHERE id = $1", self.id)
                    .fetch_optional(&mut *conn)
                    .await?
                    .is_some();

                if !exists {
                    Err(ErrorImpl::NonexistantWorkflowId(self.id))?
                }
            }
        }
    }

    pub fn read_logs<'a>(
        &'a self,
        client: &DurableClient,
    ) -> impl Stream<Item = Result<String, DurableError>> + '_ {
        let pool = client.pool.clone();

        try_stream! {
            let mut conn = pool.acquire().await?;
            let mut count = 0;

            let mut events = sqlx::query!(
                r#"
                SELECT message
                FROM durable.log
                WHERE task_id = $1
                ORDER BY index ASC
                "#,
                self.id
            )
            .fetch(&mut *conn);

            while let Some(record) = events.try_next().await? {
                count += 1;

                yield record.message;
            }

            drop(events);

            if count == 0 {
                let exists = sqlx::query!("SELECT id FROM durable.task WHERE id = $1", self.id)
                    .fetch_optional(&mut *conn)
                    .await?
                    .is_some();

                if !exists {
                    Err(ErrorImpl::NonexistantWorkflowId(self.id))?
                }
            }
        }
    }
}
