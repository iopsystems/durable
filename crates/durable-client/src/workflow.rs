use async_stream::try_stream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use serde_json::Value;
use sqlx::postgres::PgListener;
use sqlx::types::Json;

use crate::error::ErrorImpl;
use crate::event::TaskComplete;
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

    pub fn follow_logs<'a>(
        &'a self,
        client: &DurableClient,
    ) -> impl Stream<Item = Result<String, DurableError>> + '_ {
        let pool = client.pool.clone();

        try_stream!({
            let mut last_seen = -1;
            let mut listener = PgListener::connect_with(&pool).await?;
            listener
                .listen_all(["durable:log", "durable:task-complete"])
                .await?;

            loop {
                let event = listener.try_recv().await?;
                let results = sqlx::query!(
                    "
                    SELECT message, index
                     FROM durable.log
                    WHERE task_id = $1
                      AND index > $2
                    ORDER BY index ASC
                    ",
                    self.id,
                    last_seen
                )
                .fetch(&mut listener);

                for await result in results {
                    let record = result?;

                    yield record.message;
                    last_seen = last_seen.max(record.index);
                }

                match event.as_ref() {
                    Some(event) if event.channel() != "durable:task-complete" => continue,
                    Some(event) => match serde_json::from_str::<TaskComplete>(event.payload()) {
                        Ok(payload) if payload.id == self.id => break,
                        Ok(_) => continue,
                        Err(_) => (),
                    },
                    None => (),
                };

                let state = sqlx::query!(
                    r#"SELECT state::text as "state!" FROM durable.task WHERE id = $1"#,
                    self.id
                )
                .fetch_one(&mut listener)
                .await?
                .state;

                if state == "complete" || state == "suspended" {
                    break;
                }
            }
        })
    }
}
