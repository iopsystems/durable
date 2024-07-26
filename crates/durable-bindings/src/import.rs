use crate::types::*;

/// Immediately abort the task with an error.
#[fp_bindgen_support::fp_import_signature]
pub fn abort(message: String);

/// Print data to stdout.
#[fp_bindgen_support::fp_import_signature]
pub fn print(data: String);

/// Get access to the encoded json task data.
#[fp_bindgen_support::fp_import_signature]
pub fn task_data() -> String;

/// Get access to the task name.
#[fp_bindgen_support::fp_import_signature]
pub fn task_name() -> String;

/// Start a transaction. If this transaction has already executed to completion
/// then return the data from the last time it was executed.
///
/// Parameters:
/// - `label` - A text label that gets recorded in the event. This is used to
///             validate that events are in fact executing in the same order
///             when the workflow is restarted.
/// - `is-db` - Whether this transaction is a database transaction and should
///             reserve a database connection so that sql can be used within.
#[fp_bindgen_support::fp_import_signature]
pub async fn transaction_enter(label: String, is_db: bool) -> Option<String>;

/// Complete a transaction, saving the result of this transaction for future use.
///
/// Parameters:
/// - `data` - JSON-encoded state to save.
#[fp_bindgen_support::fp_import_signature]
pub async fn transaction_exit(data: String);
