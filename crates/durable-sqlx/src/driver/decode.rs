use std::borrow::Cow;

use sqlx::error::BoxDynError;
use sqlx::Decode;

use super::Value;
use crate::bindings as sql;
use crate::driver::Durable;

fn unexpected_nullable_type(expected: &str, value: &Value) -> BoxDynError {
    format!("expected {expected}, got {} instead", value.type_info()).into()
}

fn unexpected_nonnull_type(expected: &str, value: &Value) -> BoxDynError {
    match &value.0 {
        sql::Value::Null(_) => format!("expected non-null {expected}, got null instead").into(),
        _ => unexpected_nullable_type(expected, value),
    }
}

impl Decode<'_, Durable> for f32 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match &value.0 {
            &sql::Value::Float4(v) => Ok(v),
            _ => Err(unexpected_nonnull_type("float4", value)),
        }
    }
}

impl Decode<'_, Durable> for f64 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match &value.0 {
            &sql::Value::Float8(v) => Ok(v),
            _ => Err(unexpected_nonnull_type("float8", value)),
        }
    }
}

impl<'r> Decode<'r, Durable> for &'r [u8] {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match &value.0 {
            sql::Value::Bytea(v) => Ok(&v),
            _ => Err(unexpected_nonnull_type("bytea", value)),
        }
    }
}

impl<'r> Decode<'r, Durable> for Cow<'r, [u8]> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Durable>>::decode(value).map(Cow::Borrowed)
    }
}

impl Decode<'_, Durable> for Vec<u8> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <&[u8] as Decode<Durable>>::decode(value).map(|v| v.to_vec())
    }
}

impl<'r> Decode<'r, Durable> for &'r str {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match &value.0 {
            sql::Value::Text(v) => Ok(&v),
            _ => Err(unexpected_nonnull_type("text", value)),
        }
    }
}

impl<'r> Decode<'r, Durable> for Cow<'r, str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <&str as Decode<Durable>>::decode(value).map(Cow::Borrowed)
    }
}

impl Decode<'_, Durable> for String {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <&str as Decode<Durable>>::decode(value).map(|v| v.to_owned())
    }
}
