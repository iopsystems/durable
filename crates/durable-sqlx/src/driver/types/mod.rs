use durable_core::bindings::sql;
use sqlx::error::BoxDynError;

use crate::driver::Value;

mod int;

fn unexpected_nullable_type(expected: &str, value: &Value) -> BoxDynError {
    format!("expected {expected}, got {} instead", value.type_info()).into()
}

fn unexpected_nonnull_type(expected: &str, value: &Value) -> BoxDynError {
    match &value.0 {
        sql::Value::Null(_) => format!("expected non-null {expected}, got null instead").into(),
        _ => unexpected_nullable_type(expected, value),
    }
}
