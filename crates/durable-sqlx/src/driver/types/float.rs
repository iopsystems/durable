use std::borrow::Cow;

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

        Err(unexpected_nonnull_type("float4", value))
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

        Err(unexpected_nonnull_type("float8", value))
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

impl sqlx::Encode<'_, Durable> for &'_ [f32] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::float4_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<f32> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f32] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<[f32]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f32] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Cow<'_, [f32]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f32] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<f32> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_float4_array() {
            return Ok(value);
        }

        Err(unexpected_nonnull_type("float4[]", value))
    }
}

impl sqlx::Type<Durable> for &'_ [f32] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f32> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Vec<f32> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::float4_array()
    }
}

impl sqlx::Type<Durable> for Box<[f32]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f32> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Cow<'_, [f32]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f32> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Encode<'_, Durable> for &'_ [f64] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::float8_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<f64> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f64] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<[f64]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f64] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Cow<'_, [f64]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[f64] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<f64> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_float8_array() {
            return Ok(value);
        }

        Err(unexpected_nonnull_type("float8[]", value))
    }
}

impl sqlx::Type<Durable> for &'_ [f64] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f64> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Vec<f64> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::float8_array()
    }
}

impl sqlx::Type<Durable> for Box<[f64]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f64> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Cow<'_, [f64]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<f32> as sqlx::Type<Durable>>::type_info()
    }
}
