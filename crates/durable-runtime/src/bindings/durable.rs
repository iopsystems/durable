//! Bindings for the durable:core WIT interface.

wasmtime::component::bindgen!({
    path: "../durable/wit",
    world: "durable:core/core",
    tracing: true,
    trappable_imports: true,
    async: {
        except_imports: [
            "task-name",
            "task-data",
            "abort"
        ]
    }
});

pub use self::durable::core::*;
