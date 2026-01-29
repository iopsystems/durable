//! The worker and runtime responsible for executing durable tasks.

#[macro_use]
extern crate serde;

pub mod clock;
mod config;
pub mod entropy;
mod error;
pub mod event;
mod flag;
pub mod migrate;
pub mod plugin;
mod resource;
pub mod scheduler;
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
            default: async | trappable
        },

        exports: {
            default: async | trappable
        },
    });
}

pub use self::clock::{Clock, SystemClock};
pub use self::config::Config;
pub use self::entropy::{Entropy, SystemEntropy};
pub use self::error::TaskStatus;
pub use self::resource::{Resourceable, Resources};
pub use self::scheduler::{
    Component, NoopScheduler, ScheduleEvent, ScheduleGuard, Scheduler,
};
pub use self::task::Task;
pub use self::worker::{Worker, WorkerBuilder, WorkerHandle};
