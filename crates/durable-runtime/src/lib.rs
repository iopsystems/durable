//!

#[macro_use]
extern crate serde;

mod config;
mod error;
pub mod event;
mod flag;
pub mod plugin;
pub mod task;
pub mod util;
mod worker;

#[allow(unused_mut, unused_doc_comments)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use self::config::Config;
pub use self::error::TaskStatus;
pub use self::task::Task;
pub use self::worker::{Worker, WorkerBuilder, WorkerHandle};
