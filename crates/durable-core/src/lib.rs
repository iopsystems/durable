//!

use std::time::{Duration, SystemTime};

use serde::de::DeserializeOwned;
pub use serde_json::value::RawValue;

#[macro_use]
extern crate serde;

mod alloc;
mod start;
pub mod transaction;

#[allow(unused_imports, unused_braces)]
mod bindings {
    #[cfg(feature = "bindgen")]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    #[cfg(not(feature = "bindgen"))]
    include!("bindings.rs");

    pub use self::durable::core::core::*;
}

#[doc(inline)]
pub use crate::bindings::durable::core::core::{task_id, task_name};
pub use crate::transaction::transaction;

/// Read the JSON data that this task was created with.
pub fn task_data() -> Box<RawValue> {
    let data = crate::bindings::task_data();
    let data = data.into_boxed_str();

    // SAFETY:
    // 1. RawValue is a #[repr(transparent)] wrapper around a Box<str> so the
    //    transmute is safe on its own.
    // 2. The runtime guarantees that the task data is valid json, so this does not
    //    create an invalid RawValue instance.
    unsafe { std::mem::transmute(data) }
}

/// Immediately abort the workflow with a message.
pub fn abort(message: &str) -> ! {
    crate::transaction::maybe_txn::<_, ()>("durable::abort", || {
        eprintln!("{message}");

        std::process::exit(1);
    });

    // This line should never be reached, but if it does then we can panic with an
    // unreachable instruction.
    std::process::abort()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification<T = Box<RawValue>> {
    pub created_at: SystemTime,
    pub event: String,
    pub data: T,
}

/// Block this task until a new notification arrives, then return the
/// notification.
///
/// # Panics
/// Panics if the notification payload could not be deserialized into `T`.
/// If you want to avoid the chance that the notification may panic, you can use
/// a type that can deserialize any json value, such as [`serde_json::Value`] or
/// [`RawValue`].
pub fn notification<T>() -> Notification<T>
where
    T: DeserializeOwned,
{
    let event = crate::bindings::durable::core::notify::notification_blocking();
    let data: T = serde_json::from_str(&event.data) //
        .expect("could not deserialize notification payload");

    Notification {
        created_at: SystemTime::UNIX_EPOCH
            + Duration::new(event.created_at.seconds, event.created_at.nanoseconds),
        event: event.event,
        data,
    }
}
