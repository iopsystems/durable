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
        buf.push(Value::new(sql::Value::float4(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for f32 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(v) = value.0.as_float4() {
            return Ok(v);
        }

        if let Some(v) = value.0.as_float8() {
            return Ok(v as f32);
        }

        Err(unexpected_nonnull_type(&TypeInfo::float4(), value))
    }
}

impl sqlx::Encode<'_, Durable> for f64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::float8(*self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for f64 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(v) = value.0.as_float4() {
            return Ok(v.into());
        }

        if let Some(v) = value.0.as_float8() {
            return Ok(v);
        }

        Err(unexpected_nonnull_type(&TypeInfo::float8(), value))
    }
}

impl sqlx::Type<Durable> for f32 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::float4()
    }
}

impl sqlx::Type<Durable> for f64 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::float8()
    }
}

generic_slice_decl!(f32 => float4_array float4_array as_float4_array);
generic_slice_decl!(f64 => float8_array float8_array as_float8_array);
