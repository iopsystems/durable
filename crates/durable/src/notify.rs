//! Wait for notifications to be delivered from external systems.

#[doc(inline)]
pub use durable_core::Notification;

/// Block this workflow until a new notification arrives, and return that
/// notification.
///
/// This is meant to allow the workflow to wait on external events. The task
/// first blocks waiting on a notification (and gets suspended). Later, once a
/// relevant event occurs, an external system notifies the task and execution
/// picks up where it left off.
///
/// # Panics
/// This method panics if the notification cannot be deserialized into a value
/// of type `T`. If you want to handle deserialization failures yourself, use a
/// type which cannot fail to deserialize such as [`serde_json::Value`] or
/// [`serde_json::value::RawValue`].
pub fn wait<T>() -> Notification<T>
where
    T: serde::de::DeserializeOwned,
{
    durable_core::notification::<T>()
}
