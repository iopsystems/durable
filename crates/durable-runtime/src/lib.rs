//! The worker and runtime responsible for executing durable tasks.

#[macro_use]
extern crate serde;

mod config;
mod error;
pub mod event;
mod flag;
pub mod migrate;
pub mod plugin;
mod resource;
pub mod task;
pub mod util;
mod worker;

#[allow(
    rustdoc::invalid_html_tags,
    clippy::all,
    dead_code,
    unused_mut,
    unused_doc_comments
)]
mod bindings {
    wasmtime::component::bindgen!({
        world: "durable:core/imports",

        imports: {
            // "task-id": tracing,

            default: async | trappable
        },

        exports: {
            // Method implementations that should be sync

            // "durable:core/imports/core.task-id": exact,

            default: async | trappable
        },

        // include_generated_code_from_file: true
    });

    // include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use self::config::Config;
pub use self::error::TaskStatus;
pub use self::resource::{Resourceable, Resources};
pub use self::task::Task;
pub use self::worker::{Worker, WorkerBuilder, WorkerHandle};
