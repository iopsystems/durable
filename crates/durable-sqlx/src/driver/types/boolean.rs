use sqlx::encode::IsNull;

use super::unexpected_nonnull_type;
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, BoxDynError, Durable};

impl sqlx::Decode<'_, Durable> for bool {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match &value.0 {
            &sql::Value::Boolean(v) => Ok(v),
            _ => Err(unexpected_nonnull_type("bool", value)),
        }
    }
}

impl sqlx::Encode<'_, Durable> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Boolean(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Type<Durable> for bool {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Boolean)
    }
}
