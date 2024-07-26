//!

#[macro_use]
extern crate serde;

#[path = "bindings/mod.rs"]
mod bindings;
mod config;
mod error;
pub mod event;
pub mod flag;
mod task;
mod worker;

pub use self::config::Config;
pub use self::worker::{Worker, WorkerBuilder};
