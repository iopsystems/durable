use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::types::{Json, JsonRawValue};

use super::unexpected_nonnull_type;
use crate::bindings::durable::core::sql;
use crate::driver::{Durable, TypeInfo, Value};

impl<T> sqlx::Encode<'_, Durable> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let json = serde_json::to_string(&self.0)?;
        buf.push(Value(sql::Value::jsonb(&json)));
        Ok(IsNull::No)
    }
}

impl<T> sqlx::Decode<'_, Durable> for Json<T>
where
    T: DeserializeOwned,
{
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(json) = value.0.as_json() {
            let value: T = serde_json::from_str(&json)?;

            return Ok(Json(value));
        }

        Err(unexpected_nonnull_type("jsonb", value))
    }
}

impl<T> sqlx::Type<Durable> for Json<T> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb()
    }
}

impl<T> sqlx::Encode<'_, Durable> for &'_ [Json<T>]
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let mut json = Vec::with_capacity(self.len());

        for value in self.iter() {
            json.push(serde_json::to_string(value)?);
        }

        buf.push(Value(sql::Value::jsonb_array(&json)));
        Ok(IsNull::No)
    }
}

impl<T> sqlx::Encode<'_, Durable> for Vec<Json<T>>
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[Json<T>] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl<T> sqlx::Decode<'_, Durable> for Vec<Json<T>>
where
    T: DeserializeOwned,
{
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(values) = value.0.as_json_array() {
            let mut json: Self = Vec::with_capacity(values.len());

            for value in values {
                json.push(Json(serde_json::from_str(&value)?));
            }

            return Ok(json);
        }

        Err(unexpected_nonnull_type("jsonb[]", value))
    }
}

impl<T> sqlx::Type<Durable> for &'_ [Json<T>] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb_array()
    }
}

impl<T> sqlx::Type<Durable> for Vec<Json<T>> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb_array()
    }
}

impl sqlx::Encode<'_, Durable> for JsonRawValue {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <Json<_> as sqlx::Encode<Durable>>::encode(Json(self), buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<JsonRawValue> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <Json<_> as sqlx::Encode<Durable>>::encode(Json(&**self), buf)
    }
}

impl sqlx::Decode<'_, Durable> for Box<JsonRawValue> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(json) = value.0.as_json() {
            let _: &JsonRawValue = serde_json::from_str(&json)?;
            let json: Box<str> = json.into_boxed_str();

            // SAFETY: JsonRawValue is a repr(transparent) wrapper around a str. This is not
            //         necessarily a guarantee of the serde_json API but seems unlikely to
            //         change anytime soon given that serde_json takes advantage of it
            //         internally.
            let json: Box<JsonRawValue> = unsafe { std::mem::transmute(json) };

            return Ok(json);
        }

        Err(unexpected_nonnull_type("jsonb", value))
    }
}
