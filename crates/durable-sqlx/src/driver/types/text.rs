use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, Durable};

impl sqlx::Encode<'_, Durable> for &str {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::text(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Cow<'_, str> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<str> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for String {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl<'r> sqlx::Decode<'r, Durable> for Cow<'r, str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <String as sqlx::Decode<Durable>>::decode(value).map(Cow::Owned)
    }
}

impl sqlx::Decode<'_, Durable> for Box<str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <String as sqlx::Decode<Durable>>::decode(value).map(|s| s.into_boxed_str())
    }
}

impl sqlx::Decode<'_, Durable> for String {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.0.as_text() {
            return Ok(text);
        }

        Err(unexpected_nonnull_type("text", value))
    }
}

impl sqlx::Type<Durable> for String {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::text()
    }
}

impl sqlx::Type<Durable> for &'_ str {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Box<str> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Cow<'_, str> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Encode<'_, Durable> for &'_ [String] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        buf.push(Value::new(sql::Value::text_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<String> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        <&[String] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<[String]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        <&[String] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Cow<'_, [String]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError>
    where
        Self: Sized,
    {
        <&[String] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<String> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_text_array() {
            return Ok(value);
        }

        Err(unexpected_nonnull_type("text[]", value))
    }
}
