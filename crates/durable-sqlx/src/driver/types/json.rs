use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::types::Json;

use super::{encode_by_ref, unexpected_nonnull_type};
use crate::bindings::durable::core::sql;
use crate::driver::{Durable, TypeInfo, Value};

impl<T> sqlx::Encode<'_, Durable> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
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

        Err(unexpected_nonnull_type(&TypeInfo::jsonb(), value))
    }
}

impl<T> sqlx::Type<Durable> for Json<T> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb()
    }
}

impl<T: Serialize> sqlx::Encode<'_, Durable> for [Json<T>] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        let mut json = Vec::with_capacity(self.len());

        for value in self.iter() {
            json.push(serde_json::to_string(value)?);
        }

        let json: Vec<&str> = json.iter().map(|x| x.as_str()).collect();
        buf.push(Value(sql::Value::jsonb_array(&json)));
        Ok(IsNull::No)
    }
}

impl<T: Serialize> sqlx::Encode<'_, Durable> for &'_ [Json<T>] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        encode_by_ref::<[Json<T>]>(self, buf)
    }
}

impl<T> sqlx::Encode<'_, Durable> for Vec<Json<T>>
where
    T: Serialize,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
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

        Err(unexpected_nonnull_type(&TypeInfo::jsonb(), value))
    }
}

impl<T> sqlx::Type<Durable> for [Json<T>] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb_array()
    }
}

impl<T> sqlx::Type<Durable> for Vec<Json<T>> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::jsonb_array()
    }
}

// `Encode`/`Decode` for `JsonRawValue`, `&JsonRawValue` and `Box<JsonRawValue>` are
// provided by blanket impls in sqlx 0.9 that delegate to `Json<&Self>` / `Json<T>`,
// which route through the `Json` impls above, so we no longer define them here.
