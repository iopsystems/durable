use std::path::Path;

use fp_bindgen::prelude::*;

fp_import! {
    /// Get access to the task name.
    fn task_name() -> String;

    /// Get access to the encoded json task data.
    fn task_data() -> String;

    /// Immediately abort the task with an error.
    fn abort(message: String);

    /// Start a transaction. If this transaction has already executed to completion
    /// then return the data from the last time it was executed.
    ///
    /// Parameters:
    /// - `label` - A text label that gets recorded in the event. This is used to
    ///             validate that events are in fact executing in the same order
    ///             when the workflow is restarted.
    /// - `is-db` - Whether this transaction is a database transaction and should
    ///             reserve a database connection so that sql can be used within.
    async fn transaction_enter(label: String, is_db: bool) -> Option<String>;

    /// Complete a transaction, saving the result of this transaction for future use.
    ///
    /// Parameters:
    /// - `data` - JSON-encoded state to save.
    async fn transaction_exit(data: String);

    // Impure functions.
    //
    // It is only valid to call these when within a transaction. Attempting to
    // call them otherwise will immediately abort the workflow.

    /// Print data to stdout.
    fn print(data: String);
}

fp_export! {
    // fn durable_realloc()
}

fn gen_guest() {
    let config = RustPluginConfig::builder()
        .version(env!("CARGO_PKG_VERSION"))
        .name("durable-bindings")
        .build();

    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::RustPlugin(config),
        path: "crates/durable-bindings"
    });
}

fn gen_host() {
    fp_bindgen!(BindingConfig {
        bindings_type: BindingsType::RustWasmer2Runtime,
        path: "crates/durable-runtime/src/bindings"
    });
}

fn main() {
    gen_guest();
    gen_host();
}
