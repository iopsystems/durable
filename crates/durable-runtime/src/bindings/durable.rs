//! Bindings for the durable:core WIT interface.

wasmtime::component::bindgen!({
    path: "../durable-core/wit",
    world: "durable:core/core",
    tracing: true,
    trappable_imports: true,
    async: {
        except_imports: [
            "task-id",
            "task-name",
            "task-data",
            "abort"
        ]
    }
});

pub use self::durable::core::*;
