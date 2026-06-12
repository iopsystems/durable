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
                let mut tx = self.state.pool().begin().await?;

                self.state
                    .storage()
                    .suspend_task_no_wakeup(&mut tx, self.task_id())
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
    ) -> anyhow::Result<Option<Event>> {
        if self.state.transaction().is_some() {
            anyhow::bail!(
                "durable:core/notify.notification-blocking-timeout cannot be called from within a \
                 transaction"
            );
        }

        let timeout = std::time::Duration::from_nanos(timeout_ns);

        // Durably record the absolute deadline as a recorded event *before* the
        // result. This is what gives the timed wait a timer fallback: it is
        // - computed from the injected `Clock` (so a `DstClock` controls it), and
        // - persisted, so it survives a suspend/replay cycle. On replay we read
        //   the recorded value back rather than recomputing a fresh deadline.
        let deadline_options =
            TransactionOptions::new("durable:core/notify.notification-blocking-timeout.deadline");
        let deadline: DateTime<Utc> =
            match self.state.enter::<DateTime<Utc>>(deadline_options).await? {
                Some(deadline) => deadline,
                None => {
                    let deadline = chrono::Duration::from_std(timeout)
                        .ok()
                        .and_then(|d| self.state.clock().now().checked_add_signed(d))
                        .unwrap_or(DateTime::<Utc>::MAX_UTC);
                    self.state.exit(&deadline).await?;
                    deadline
                }
            };

        let options = TransactionOptions::new("durable:core/notify.notification-blocking-timeout");
        if let Some(event) = self.state.enter::<Option<EventData>>(options).await? {
            return Ok(event.map(Into::into));
        }

        let suspend_timeout = self.state.config().suspend_timeout;
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

            // Compute the time remaining until the user deadline using the
            // injected clock. If it has already elapsed (e.g. we were revived by
            // the wakeup timer at `deadline`), `user_deadline` is now and the
            // select below resolves the timeout immediately.
            let remaining = (deadline - self.state.clock().now())
                .to_std()
                .unwrap_or(std::time::Duration::ZERO);
            let user_deadline = Instant::now() + remaining;
            let suspend_deadline = Instant::now() + suspend_timeout;

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

                // The suspend timeout expired. Suspend the task to free up the
                // worker slot, recording `deadline` as the wakeup time so the
                // task is revived by the timer even if its notification is never
                // re-delivered.
                Some(Expired::Suspend) => {
                    let mut tx = self.state.pool().begin().await?;

                    self.state
                        .storage()
                        .suspend_task(&mut tx, self.task_id(), Some(deadline))
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
