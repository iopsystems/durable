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
mod util;
mod worker;

#[allow(rustdoc::invalid_html_tags, unused_mut, unused_doc_comments)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use self::config::Config;
pub use self::error::TaskStatus;
pub use self::resource::{Resourceable, Resources};
pub use self::task::Task;
pub use self::worker::{Worker, WorkerBuilder, WorkerHandle};
