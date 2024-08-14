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
        buf.push(Value(sql::Value::Int1(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i16 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Int2(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Int4(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for i64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Int8(*self)));
        Ok(IsNull::No)
    }
}

fn decode_int(value: &Value) -> Result<i64, BoxDynError> {
    match &value.0 {
        &sql::Value::Int1(v) => Ok(v.into()),
        &sql::Value::Int2(v) => Ok(v.into()),
        &sql::Value::Int4(v) => Ok(v.into()),
        &sql::Value::Int8(v) => Ok(v),
        _ => Err(unexpected_nonnull_type("integer", value)),
    }
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
        TypeInfo(sql::PrimitiveType::Int1)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Int1
                | sql::PrimitiveType::Int2
                | sql::PrimitiveType::Int4
                | sql::PrimitiveType::Int8
        )
    }
}

impl sqlx::Type<Durable> for i16 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Int2)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Int1
                | sql::PrimitiveType::Int2
                | sql::PrimitiveType::Int4
                | sql::PrimitiveType::Int8
        )
    }
}

impl sqlx::Type<Durable> for i32 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Int4)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Int1
                | sql::PrimitiveType::Int2
                | sql::PrimitiveType::Int4
                | sql::PrimitiveType::Int8
        )
    }
}

impl sqlx::Type<Durable> for i64 {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Int8)
    }

    fn compatible(ty: &<Durable as sqlx::Database>::TypeInfo) -> bool {
        matches!(
            ty.0,
            sql::PrimitiveType::Int1
                | sql::PrimitiveType::Int2
                | sql::PrimitiveType::Int4
                | sql::PrimitiveType::Int8
        )
    }
}
