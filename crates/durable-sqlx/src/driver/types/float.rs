use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, Durable};

impl sqlx::Encode<'_, Durable> for f32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Float4(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for f32 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match &value.0 {
            &sql::Value::Float4(v) => Ok(v),
            &sql::Value::Float8(v) => Ok(v as f32),
            _ => Err(unexpected_nonnull_type("float4", value)),
        }
    }
}

impl sqlx::Encode<'_, Durable> for f64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Float8(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for f64 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match &value.0 {
            &sql::Value::Float8(v) => Ok(v),
            &sql::Value::Float4(v) => Ok(v.into()),
            _ => Err(unexpected_nonnull_type("float8", value)),
        }
    }
}

impl sqlx::Type<Durable> for f32 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Float4)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Float4 | sql::PrimitiveType::Float8
        )
    }
}

impl sqlx::Type<Durable> for f64 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Float8)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Float4 | sql::PrimitiveType::Float8
        )
    }
}
