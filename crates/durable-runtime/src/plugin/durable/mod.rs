//! Plugins for built-in runtime functionality.

use wasmtime::component::Linker;

use crate::bindings::Imports;
use crate::plugin::{Plugin, Task};

mod core;
mod http;
mod sql;

pub struct DurablePlugin;

impl Plugin for DurablePlugin {
    fn name(&self) -> &str {
        "durable:core"
    }

    fn setup(&self, linker: &mut Linker<Task>, _: &mut Task) -> wasmtime::Result<()> {
        Imports::add_to_linker(linker, |task| task)
    }
}
