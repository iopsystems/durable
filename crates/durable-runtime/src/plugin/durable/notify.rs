use chrono::{DateTime, Utc};
use serde_json::value::RawValue;
use sqlx::types::Json;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;

use crate::bindings::durable::core::notify::{Event, Host, NotifyError};
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

    async fn notification_blocking_timeout(
        &mut self,
        timeout_ns: u64,
    ) -> wasmtime::Result<Option<Event>> {
        if self.state.transaction().is_some() {
            anyhow::bail!(
                "durable:core/notify.notification-blocking-timeout cannot be called from within a \
                 transaction"
            );
        }

        let options = TransactionOptions::new("durable:core/notify.notification-blocking-timeout");
        if let Some(event) = self.state.enter::<Option<EventData>>(options).await? {
            return Ok(event.map(Into::into));
        }

        let timeout = std::time::Duration::from_nanos(timeout_ns);
        let user_deadline = Instant::now() + timeout;
        let suspend_deadline = Instant::now() + self.state.config().suspend_timeout;
        let task_id = self.state.task_id();
        let mut rx = self.state.subscribe_notifications();

        let data = loop {
            let mut tx = self.state.pool().begin().await?;
            let data = poll_notification(&mut *self, &mut tx).await?;

            if let Some(data) = data {
                let txn = self.state.transaction_mut().unwrap();
                txn.set_conn(tx)?;

                break Some(data);
            }

            tx.rollback().await?;

            // Wait for either a notification, the user timeout, or the suspend
            // timeout — whichever comes first.
            enum Expired {
                User,
                Suspend,
            }

            let expired = 'inner: loop {
                tokio::select! {
                    biased;

                    result = rx.recv() => match result {
                        Ok(notif) if notif.task_id == task_id => break 'inner None,
                        Ok(_) => continue 'inner,
                        Err(RecvError::Lagged(_)) => break 'inner None,
                        Err(RecvError::Closed) => {
                            return Err(anyhow::Error::new(TaskStatus::NotScheduledOnWorker))
                        }
                    },
                    _ = tokio::time::sleep_until(user_deadline) => {
                        break 'inner Some(Expired::User)
                    },
                    _ = tokio::time::sleep_until(suspend_deadline) => {
                        break 'inner Some(Expired::Suspend)
                    },
                }
            };

            match expired {
                // A notification signal arrived — go back to the top and poll.
                None => continue,

                // The user's timeout expired. Check one more time for a
                // notification that may have arrived concurrently.
                Some(Expired::User) => {
                    let mut tx = self.state.pool().begin().await?;
                    let data = poll_notification(&mut *self, &mut tx).await?;

                    if let Some(data) = data {
                        let txn = self.state.transaction_mut().unwrap();
                        txn.set_conn(tx)?;
                        break Some(data);
                    }

                    tx.rollback().await?;
                    break None;
                }

                // The suspend timeout expired. Attempt to suspend the task so
                // we free up the worker slot, just like notification_blocking.
                Some(Expired::Suspend) => {
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
                        // A new notification barged in while we were updating.
                        // Roll back the suspend and go through the main loop.
                        tx.rollback().await?;
                        continue;
                    }

                    tx.commit().await?;
                    return Err(anyhow::Error::new(TaskStatus::Suspend));
                }
            }
        };

        self.exit(&data).await?;

        Ok(data.map(Into::into))
    }

    async fn notify(
        &mut self,
        task: i64,
        event: String,
        data: String,
    ) -> wasmtime::Result<Result<(), NotifyError>> {
        if self.state.transaction().is_some() {
            anyhow::bail!("durable:core/notify.notify cannot be called from within a transaction");
        }

        let options = TransactionOptions::new("durable:core/notify.notify").database(true);
        if let Some(result) = self.state.enter::<Result<(), NotifyError>>(options).await? {
            return Ok(result);
        }

        let txn = self.state.transaction_mut().unwrap();
        let tx = txn.conn().unwrap();

        let future = async {
            let json: &RawValue = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(e) => return Ok(Err(NotifyError::Other(e.to_string()))),
            };

            // Note: We lock the row here so that concurrent notification polls
            //       cannot barge in here.
            let state = sqlx::query_scalar!(
                r#"
                SELECT state as "state!: TaskState" 
                 FROM durable.task
                WHERE task.id = $1
                FOR UPDATE
                "#,
                task
            )
            .fetch_optional(&mut **tx)
            .await?;

            match state {
                Some(TaskState::Complete | TaskState::Failed) => {
                    return Ok(Err(NotifyError::TaskDead))
                }
                None => return Ok(Err(NotifyError::TaskNotFound)),
                _ => (),
            }

            let result = sqlx::query_scalar!(
                r#"
                INSERT INTO durable.notification(task_id, event, data)
                VALUES ($1, $2, $3)
                "#,
                task,
                event,
                Json(json) as Json<&RawValue>
            )
            .execute(&mut **tx)
            .await;

            match result {
                Ok(_) => Ok(Ok(())),
                Err(sqlx::Error::Database(ref error)) if error.constraint() == Some("fk_task") => {
                    Ok(Err(NotifyError::TaskNotFound))
                }
                Err(e) => Err(e),
            }
        };

        let result = future.await?;
        self.state.exit(&result).await?;

        Ok(result)
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

#[derive(Serialize, Deserialize)]
#[serde(remote = "NotifyError")]
#[serde(tag = "error", content = "message")]
#[serde(rename_all = "kebab-case")]
enum RemoteNotifyError {
    TaskNotFound,
    TaskDead,
    Other(String),
}

impl serde::Serialize for NotifyError {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        RemoteNotifyError::serialize(self, ser)
    }
}

impl<'de> serde::Deserialize<'de> for NotifyError {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        RemoteNotifyError::deserialize(de)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "durable.task_state", rename_all = "lowercase")]
enum TaskState {
    Ready,
    Active,
    Suspended,
    Complete,
    Failed,
}
