//!

pub use serde_json::value::RawValue;

#[macro_use]
extern crate serde;

mod start;
pub mod transaction;

pub mod bindings {
    include!("bindings.rs");

    #[allow(unused_imports)]
    pub use self::durable::core::*;
}

pub use crate::transaction::transaction;

pub mod export {
    pub use crate::start::durable_start;
}

/// The numeric id of the currently executing task.
pub fn task_id() -> i64 {
    crate::bindings::task_id()
}

/// The name of the currently executing task.
///
/// This is the name that was provided when the task was being created.
pub fn task_name() -> String {
    crate::bindings::task_name()
}

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
    crate::bindings::abort(message);

    // SAFETY: The abort function will never return.
    unsafe { std::hint::unreachable_unchecked() }
}

pub fn print(message: &str) {
    crate::transaction::maybe_txn("durable::print", || {
        crate::bindings::print(message);
        message.to_owned()
    });
}
