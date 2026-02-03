//! Wait for notifications to be delivered from external systems.
//!
//! Having each and every task try to poll external systems for event completion
//! is inefficient and liable to result in resource exhaustion either within the
//! runtime or in external systems.
//!
//! Instead, durable provides notifications as a way for tasks to be woken up on
//! external events. To do so, a task calls [`wait`] which will block until a
//! notification is sent to this task. Once an event occurs, the external system
//! just needs to arrange for a notification to be posted to the right task, and
//! it will pick up where it left off.

#[doc(inline)]
pub use durable_core::notify::{Notification, NotifyError, NotifyErrorKind};
use serde::Serialize;
use std::time::Duration;

/// Block this workflow until a new notification arrives, and return that
/// notification.
///
/// This is meant to allow the workflow to wait on external events. The task
/// first blocks waiting on a notification (and gets suspended). Later, once a
/// relevant event occurs, an external system notifies the task and execution
/// picks up where it left off.
///
/// # Traps
/// Attempting to call this function within a transaction will result in a trap
/// that instantly kills the workflow.
pub fn wait() -> Notification {
    durable_core::notify::wait()
}

/// Block this workflow until a new notification arrives or the timeout expires.
///
/// Returns `Some(notification)` if a notification was received before the
/// timeout, or `None` if the timeout expired without receiving a notification.
///
/// # Traps
/// Attempting to call this function within a transaction will result in a trap
/// that instantly kills the workflow.
pub fn wait_with_timeout(timeout: Duration) -> Option<Notification> {
    durable_core::notify::wait_with_timeout(timeout)
}

/// Send a notification to another durable task.
///
/// # Errors
/// This function will return an error if:
/// - The requested task does not exist.
/// - The requested task has already completed.
/// - `data` cannot be serialized to JSON.
///
/// # Traps
/// Attempting to call this function within a transaction will result in a trap
/// that instantly kills the workflow.
pub fn notify<T>(task: i64, event: &str, data: &T) -> Result<(), NotifyError>
where
    T: ?Sized + Serialize,
{
    durable_core::notify::notify(task, event, data)
}
