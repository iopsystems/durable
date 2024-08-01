//!

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
    crate::bindings::durable::core::core::abort(message);

    // SAFETY: The abort function will never return.
    unsafe { std::hint::unreachable_unchecked() }
}
