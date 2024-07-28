//!

pub use serde_json::value::RawValue;

#[cfg(feature = "http")]
#[doc(inline)]
pub extern crate durable_http as http;
#[cfg(feature = "sqlx")]
pub extern crate durable_sqlx as sqlx;

#[doc(inline)]
pub use durable_core::{abort, print, task_data, task_id, task_name, transaction::transaction};

#[macro_export]
macro_rules! durable_main {
    ($main:path) => {
        #[no_mangle]
        fn _start() {
            $crate::export::durable_start($main)
        }
    };
}

#[doc(hidden)]
pub mod export {
    pub use durable_core::export::*;
}
