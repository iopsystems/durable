//! Notification support for durable.

use std::fmt;
use std::time::{Duration, SystemTime};

use serde_json::value::RawValue;

use crate::bindings::durable::core::notify;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub created_at: SystemTime,
    pub event: String,
    pub data: Box<RawValue>,
}

impl Notification {
    /// Deserialize the notification payload as a json value of type `T`.
    pub fn json<'de, T>(&'de self) -> serde_json::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        serde_json::from_str(self.data.get())
    }
}

/// Block this task until a new notification arrives, then return the
/// notification.
///
/// # Traps
/// Attempting to call this function within a transaction will result in a trap
/// that instantly kills the workflow.
pub fn wait() -> Notification {
    let event = crate::bindings::durable::core::notify::notification_blocking();
    let data = event.data.into_boxed_str();

    let _: &RawValue = serde_json::from_str(&data).expect(
        "durable:core/notify.notification_blocking returned an event containing invalid json data",
    );

    // SAFETY:
    // - RawValue is a #[repr(transparent)] wrapper around str, so the transmute is
    //   safe. This is also unlikely to change in future serde versions.
    // - We have just validated that event.data is valid json, so this doesn't break
    //   RawValue's invariants.
    let data = unsafe { std::mem::transmute::<Box<str>, Box<RawValue>>(data) };

    Notification {
        created_at: SystemTime::UNIX_EPOCH
            + Duration::new(event.created_at.seconds, event.created_at.nanoseconds),
        event: event.event,
        data,
    }
}

/// Block this task until a new notification arrives or the timeout expires.
///
/// Returns `Some(notification)` if a notification was received, or `None` if
/// the timeout expired without receiving a notification.
///
/// # Traps
/// Attempting to call this function within a transaction will result in a trap
/// that instantly kills the workflow.
pub fn wait_with_timeout(timeout: Duration) -> Option<Notification> {
    let timeout_ns = timeout.as_nanos().min(u64::MAX as u128) as u64;
    let event =
        crate::bindings::durable::core::notify::notification_blocking_timeout(timeout_ns);

    event.map(|event| {
        let data = event.data.into_boxed_str();

        let _: &RawValue = serde_json::from_str(&data).expect(
            "durable:core/notify.notification_blocking_timeout returned an event containing \
             invalid json data",
        );

        // SAFETY: Same as in wait() - RawValue is #[repr(transparent)] around str.
        let data = unsafe { std::mem::transmute::<Box<str>, Box<RawValue>>(data) };

        Notification {
            created_at: SystemTime::UNIX_EPOCH
                + Duration::new(event.created_at.seconds, event.created_at.nanoseconds),
            event: event.event,
            data,
        }
    })
}

/// Send a notification to another task.
///
/// # Errors
/// This method will emit an error if
/// - The requested task does not exist.
/// - The requested task has already completed.
/// - The provided data is not valid JSON.
///
/// # Traps
/// Attempting to call this function from within a transaction will result in a
/// trap that instantly kills the workflow.
pub fn notify<T>(task: i64, event: &str, data: &T) -> Result<(), NotifyError>
where
    T: ?Sized + serde::Serialize,
{
    let data =
        serde_json::to_string(&data).map_err(|e| NotifyError(ErrorData::Serialization(e)))?;
    notify::notify(task, event, &data).map_err(|e| NotifyError(ErrorData::Bindings(e)))
}

/// Errors that can occur when attempting to notify another task.
pub struct NotifyError(ErrorData);

impl NotifyError {
    pub fn kind(&self) -> NotifyErrorKind {
        use notify::NotifyError::*;

        match &self.0 {
            ErrorData::Bindings(TaskNotFound) => NotifyErrorKind::TaskNotFound,
            ErrorData::Bindings(TaskDead) => NotifyErrorKind::TaskDead,
            _ => NotifyErrorKind::Other,
        }
    }
}

/// An enum listing out the types of errors that can occur while attempting to
/// send a notification to another task.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotifyErrorKind {
    /// There was no task with the requested task ID.
    TaskNotFound,
    /// The task exists but is no longer executing.
    TaskDead,

    /// Another error outside of those listed here.
    Other,
}

enum ErrorData {
    Bindings(notify::NotifyError),
    Serialization(serde_json::Error),
}

impl fmt::Debug for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorData::Bindings(err) => err.fmt(f),
            ErrorData::Serialization(err) => f.debug_tuple("Serialization").field(err).finish(),
        }
    }
}

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use notify::NotifyError::*;

        match &self.0 {
            ErrorData::Bindings(TaskNotFound) => write!(f, "no task exists with the requested id"),
            ErrorData::Bindings(TaskDead) => {
                write!(f, "unable to notify a task that has already completed")
            }
            ErrorData::Bindings(Other(msg)) => f.write_str(msg),
            ErrorData::Serialization(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for NotifyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorData::Serialization(err) => Some(err),
            _ => None,
        }
    }
}
