use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::{Decode, Encode};

use super::unexpected_nonnull_type;
use crate::bindings as sql;
use crate::driver::{Durable, TypeInfo, Value};

impl Encode<'_, Durable> for i8 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::int1(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i16 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::int2(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::int4(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::int8(*self)));
        Ok(IsNull::No)
    }
}

fn decode_int(value: &Value) -> Result<i64, BoxDynError> {
    if let Some(v) = value.0.as_int1() {
        return Ok(v.into());
    }

    if let Some(v) = value.0.as_int2() {
        return Ok(v.into());
    }

    if let Some(v) = value.0.as_int4() {
        return Ok(v.into());
    }

    if let Some(v) = value.0.as_int8() {
        return Ok(v);
    }

    Err(unexpected_nonnull_type(&TypeInfo::int8(), value))
}

impl Decode<'_, Durable> for i8 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int(value)?
            .try_into()
            .map_err(|_| "integer value out of range".into())
    }
}

impl Decode<'_, Durable> for i16 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int(value)?
            .try_into()
            .map_err(|_| "integer value out of range".into())
    }
}

impl Decode<'_, Durable> for i32 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int(value)?
            .try_into()
            .map_err(|_| "integer value out of range".into())
    }
}

impl Decode<'_, Durable> for i64 {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        decode_int(value)
    }
}

impl sqlx::Type<Durable> for i8 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::int1()
    }
}

impl sqlx::Type<Durable> for i16 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::int2()
    }
}

impl sqlx::Type<Durable> for i32 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::int4()
    }
}

impl sqlx::Type<Durable> for i64 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::int8()
    }
}

generic_slice_decl!(i8  => int1_array int1_array as_int1_array);
generic_slice_decl!(i16 => int2_array int2_array as_int2_array);
generic_slice_decl!(i32 => int4_array int4_array as_int4_array);
generic_slice_decl!(i64 => int8_array int8_array as_int8_array);
