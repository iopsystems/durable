pub use serde_json::value::RawValue;

#[macro_use]
extern crate serde;

mod alloc;
pub mod notify;
mod start;
pub mod transaction;

#[allow(unused_imports, unused_braces, clippy::all)]
mod bindings {
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
