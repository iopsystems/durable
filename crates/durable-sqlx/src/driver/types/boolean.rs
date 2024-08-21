use sqlx::encode::IsNull;

use super::unexpected_nonnull_type;
use crate::bindings::durable::core::sql;
use crate::driver::{TypeInfo, Value};
use crate::{BoxDynError, Durable};

impl sqlx::Decode<'_, Durable> for bool {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.0.as_boolean() {
            Some(v) => Ok(v),
            _ => Err(unexpected_nonnull_type("bool", value)),
        }
    }
}

impl sqlx::Encode<'_, Durable> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::boolean(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Type<Durable> for bool {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::boolean()
    }
}

impl sqlx::Decode<'_, Durable> for Vec<bool> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.0.as_boolean_array() {
            Some(v) => Ok(v),
            _ => Err(unexpected_nonnull_type("bool[]", value)),
        }
    }
}

impl sqlx::Encode<'_, Durable> for Vec<bool> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::boolean_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Type<Durable> for Vec<bool> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::boolean_array()
    }
}
