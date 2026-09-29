use chrono::{DateTime, Utc};
use serde_json::value::RawValue;
use sqlx::types::Json;
use tokio::sync::broadcast::error::RecvError;
use tokio::time::Instant;

use crate::bindings::durable::core::notify::{Event, Host, NotifyError};
use crate::storage::TaskState;
use crate::task::TransactionOptions;
use crate::{Task, TaskStatus};

async fn poll_notification(
    task: &mut Task,
    tx: &mut sqlx::PgConnection,
) -> anyhow::Result<Option<EventData>> {
    let task_id = task.state.task_id();
    let data = task
        .state
        .storage()
        .poll_notification(&mut *tx, task_id)
        .await?
        .map(|n| EventData {
            created_at: n.created_at,
            event: n.event,
            data: n.data,
        });

    Ok(data)
}

/// Try to suspend a task that is waiting on a notification.
///
/// Returns `true` if the task was suspended and `false` if a notification
/// was pending, in which case nothing was changed and the caller should go
/// back to polling.
///
/// The task row is locked `FOR UPDATE` before anything else. A notification
/// inserted by another transaction holds `FOR KEY SHARE` on the task row from
/// its `fk_task` check until that transaction commits, and `FOR UPDATE`
/// conflicts with it:
///
/// - If the insert happened first, this waits for it to commit. The poll below
///   then runs with a new snapshot, sees the notification, and the suspend is
///   abandoned.
/// - If this lock is taken first, the insert waits for the suspend to commit.
///   The `notify_notification` trigger then sees `state = 'suspended'` and
///   wakes the task.
///
/// The suspend `UPDATE` alone only takes `FOR NO KEY UPDATE`, which does not
/// conflict with `FOR KEY SHARE`. Without the explicit lock, a notification
/// whose trigger ran while the task was still active and which committed after
/// this poll was invisible to both sides, and the task stayed suspended until
/// `unwedge_stuck_notifications` swept it up.
async fn try_suspend(task: &mut Task) -> anyhow::Result<bool> {
    let task_id = task.task_id();
    let mut tx = task.state.pool().begin().await?;

    task.state
        .storage()
        .fetch_task_state_locked(&mut tx, task_id)
        .await?;

    task.state
        .storage()
        .suspend_task_no_wakeup(&mut tx, task_id)
        .await?;

    if poll_notification(&mut *task, &mut tx).await?.is_some() {
        // A notification arrived before we took the lock. Roll back so it
        // stays queued and the suspend is undone.
        tx.rollback().await?;
        return Ok(false);
    }

    tx.commit().await?;
    Ok(true)
}

impl Host for Task {
    async fn notification_blocking(&mut self) -> anyhow::Result<Event> {
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
                if !try_suspend(self).await? {
                    break 'inner;
                }

                return Err(anyhow::Error::new(TaskStatus::Suspend));
            }
        };

        self.exit(&data).await?;

        Ok(data.into())
    }

    async fn notification_blocking_timeout(
        &mut self,
        timeout_ns: u64,
    ) -> anyhow::Result<Option<Event>> {
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
                    if !try_suspend(self).await? {
                        continue;
                    }

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
    ) -> anyhow::Result<Result<(), NotifyError>> {
        if self.state.transaction().is_some() {
            anyhow::bail!("durable:core/notify.notify cannot be called from within a transaction");
        }

        let options = TransactionOptions::new("durable:core/notify.notify").database(true);
        if let Some(result) = self.state.enter::<Result<(), NotifyError>>(options).await? {
            return Ok(result);
        }

        let storage = self.state.shared.storage.clone();
        let txn = self.state.transaction_mut().unwrap();
        let tx = txn.conn().unwrap();

        let future = async {
            let json: &RawValue = match serde_json::from_str(&data) {
                Ok(value) => value,
                Err(e) => return Ok(Err(NotifyError::Other(e.to_string()))),
            };

            // Note: We lock the row here so that concurrent notification polls
            //       cannot barge in here.
            let state = storage.fetch_task_state_locked(&mut *tx, task).await?;

            match state {
                Some(TaskState::Complete | TaskState::Failed) => {
                    return Ok(Err(NotifyError::TaskDead))
                }
                None => return Ok(Err(NotifyError::TaskNotFound)),
                _ => (),
            }

            let result = storage
                .insert_notification(&mut *tx, task, &event, Json(json))
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
