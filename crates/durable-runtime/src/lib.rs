//!

#[macro_use]
extern crate serde;

mod config;
mod error;
pub mod event;
pub mod flag;
pub mod plugin;
mod worker;

mod bindings {
    #![allow(unused_mut)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use self::config::Config;
pub use self::plugin::Task;
pub use self::worker::{Worker, WorkerBuilder};
