use wasi::WasiResources;
use wasmtime::component::{HasSelf, Linker};

use crate::task::Task;

pub mod durable;
mod util;
pub mod wasi;

/// A plugin for the durable runtime.
///
/// Plugins allow you to expose custom functions to workers running within the
/// WASM vm. This trait provides the hooks necessary for a plugin to set itself
/// up when a task is started.
pub trait Plugin: Send + Sync {
    /// The name of this plugin.
    ///
    /// This is used for error messages in case something goes wrong when
    /// setting up the plugin.
    fn name(&self) -> &str;

    /// Perform setup required by this plugin.
    ///
    /// This should add any functions exported by this plugin to the linker and
    /// setup any state this plugin needs within the task plugin data.
    fn setup(&self, linker: &mut Linker<Task>, store: &mut Task) -> wasmtime::Result<()>;
}

pub use self::util::PluginMapExt;

pub struct DurablePlugin;

impl Plugin for DurablePlugin {
    fn name(&self) -> &str {
        "durable:core"
    }

    fn setup(&self, linker: &mut Linker<Task>, task: &mut Task) -> wasmtime::Result<()> {
        use crate::bindings::Imports;

        task.plugins.insert(WasiResources::new());
        Imports::add_to_linker::<_, HasSelf<_>>(linker, |task| task)?;
        Ok(())
    }
}
