use chrono::{DateTime, Utc};
use serde_json::value::RawValue;
use sqlx::types::Json;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;

use crate::bindings::durable::core::notify::{Event, Host};
use crate::task::TransactionOptions;
use crate::{Task, TaskStatus};

async fn poll_notification(
    task: &mut Task,
    tx: &mut sqlx::PgConnection,
) -> anyhow::Result<Option<EventData>> {
    let data = sqlx::query_as!(
        EventData,
        r#"
        DELETE FROM durable.notification
        WHERE ctid IN (
            SELECT ctid
             FROM durable.notification
            WHERE task_id = $1
            ORDER BY created_at ASC
            LIMIT 1
            FOR UPDATE
        )
        RETURNING
            created_at,
            event,
            data as "data: Json<Box<RawValue>>"
        "#,
        task.state.task_id()
    )
    .fetch_optional(&mut *tx)
    .await?;

    Ok(data)
}

#[async_trait::async_trait]
impl Host for Task {
    async fn notification_blocking(&mut self) -> wasmtime::Result<Event> {
        if self.state.transaction().is_some() {
            anyhow::bail!(
                "durable:core/notify.notification-blocking cannot be called from within a \
                 transaction"
            );
        }

        let options = TransactionOptions::new("durable:core/notify.notification-blocking");
        if let Some(event) = self.state.enter::<EventData>(options).await? {
            return Ok(event.into());
        }

        let deadline = Instant::now() + self.state.config().suspend_timeout;
        let task_id = self.state.task_id();
        let mut rx = self.state.subscribe_notifications();

        let data = loop {
            let mut tx = self.state.pool().begin().await?;
            let data = poll_notification(&mut *self, &mut tx).await?;

            if let Some(data) = data {
                let txn = self.state.transaction_mut().unwrap();
                txn.set_conn(tx)?;

                break data;
            }

            tx.rollback().await?;

            'inner: loop {
                tokio::select! {
                    biased;

                    result = rx.recv() => match result {
                        Ok(notif) if notif.task_id == task_id => break 'inner,
                        Ok(_) => continue 'inner,
                        Err(RecvError::Lagged(_)) => break 'inner,
                        Err(RecvError::Closed) => {
                            return Err(anyhow::Error::new(TaskStatus::NotScheduledOnWorker))
                        }
                    },
                    _ = tokio::time::sleep_until(deadline) => ()
                }

                // The timer expired, so we need to attempt to suspend.
                let mut tx = self.state.pool().begin().await?;

                sqlx::query!(
                    "UPDATE durable.task
                      SET state = 'suspended',
                          running_on = NULL
                    WHERE id = $1
                    ",
                    self.task_id()
                )
                .execute(&mut *tx)
                .await?;

                if poll_notification(&mut *self, &mut tx).await?.is_some() {
                    // A new notification barged in while we were updating. Roll back the
                    // transaction and go through the main loop again.
                    tx.rollback().await?;
                    break 'inner;
                }

                // At this point the lock on the current task will block any
                // competing transactions until after we are completely
                // suspended.
                tx.commit().await?;

                return Err(anyhow::Error::new(TaskStatus::Suspend));
            }
        };

        self.exit(&data).await?;

        Ok(data.into())
    }
}

#[derive(Serialize, Deserialize)]
struct EventData {
    created_at: DateTime<Utc>,
    event: String,
    data: Json<Box<RawValue>>,
}

impl From<EventData> for Event {
    fn from(data: EventData) -> Self {
        let duration = data
            .created_at
            .signed_duration_since(DateTime::<Utc>::UNIX_EPOCH)
            .to_std()
            .unwrap_or_default();

        Self {
            created_at: duration.into(),
            event: data.event,
            data: data.data.get().to_owned(),
        }
    }
}
