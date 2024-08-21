use sqlx::error::BoxDynError;
use sqlx::Value as _;

use crate::driver::Value;

mod boolean;
mod bytea;
mod float;
mod int;
mod text;
#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "chrono")]
mod chrono;

fn unexpected_nullable_type(expected: &str, value: &Value) -> BoxDynError {
    format!("expected {expected}, got {} instead", value.type_info()).into()
}

fn unexpected_nonnull_type(expected: &str, value: &Value) -> BoxDynError {
    if value.is_null() {
        return format!("expected non-null {expected}, got null instead").into();
    }

    unexpected_nullable_type(expected, value)
}
