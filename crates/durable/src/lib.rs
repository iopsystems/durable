//! Durable task guest APIs.
//!
//! This crate provides wrappers that allow durable workflows to interact with
//! the durable runtime. It provides methods that allow a durable task to
//! interact with the outside world.
//!
//! # Quickstart
//! If you are just looking to do some things in the middle of your workflow,
//! then
//! - the [`http`] module allows you to make HTTP requests,
//! - the [`sqlx`] module allows you to make SQL queries to the database that
//!   the worker is using,
//! - the [`notify`] module allows you to wait for notifications by external
//!   services.
//!
//! Otherwise, you can get the data this task was started with via the [`Task`]
//! object.
//!
//! # Features
//! - `http` - enables the [`http`] module and everything within.
//! - `sqlx` - enables the [`sqlx`] module and everything within.

#![cfg_attr(docsrs, feature(doc_cfg))]

use serde::de::Deserialize;
pub use serde_json::value::RawValue;

#[doc(inline)]
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub extern crate durable_http as http;

#[doc(inline)]
#[cfg(feature = "sqlx")]
#[cfg_attr(docsrs, doc(cfg(feature = "sqlx")))]
pub extern crate durable_sqlx as sqlx;

mod error;
pub mod notify;

#[doc(inline)]
pub use durable_core::{abort, transaction::transaction};

pub use crate::error::{Causes, Error};

pub type Result<T> = std::result::Result<T, Error>;

/// Information about the current task.
#[derive(Clone, Debug)]
pub struct Task {
    id: i64,
    name: String,
    data: Box<RawValue>,
}

impl Task {
    /// Information about the current task.
    pub fn current() -> Self {
        Self {
            id: durable_core::task_id(),
            name: durable_core::task_name(),
            data: durable_core::task_data(),
        }
    }

    /// The database id of the current task.
    pub fn id(&self) -> i64 {
        self.id
    }

    /// The name of the current task.
    ///
    /// This is set by whoever submitted the task and is only used for log
    /// messages and debugging purposes by the runtime.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The data that this task was created with, deserialized as a `T`.
    ///
    /// # Panics
    /// This method panics if the task data could not be deserialized into a
    /// `T`. If you would like to perform the deserialization yourself, use
    /// [`raw_data`](Task::raw_data) instead.
    pub fn data<'de, T: Deserialize<'de>>(&'de self) -> T {
        match serde_json::from_str(self.data.get()) {
            Ok(data) => data,
            Err(e) => panic!("failed to deserialize task data: {e}"),
        }
    }

    /// The data that this task was created with.
    ///
    /// This is guaranteed to be valid JSON. The contents of that JSON could be
    /// anything, though.
    pub fn raw_data(&self) -> &RawValue {
        &self.data
    }
}

/// Fetch information about the current durable task.
///
/// This is a convenience alias for [`Task::current`].
pub fn task() -> Task {
    Task::current()
}
