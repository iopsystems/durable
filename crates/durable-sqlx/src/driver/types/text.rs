use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, Durable};

impl sqlx::Encode<'_, Durable> for str {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::text(self)));
        Ok(IsNull::No)
    }
}

forward_encode_deref!(&'_ str => str);
forward_encode_deref!(Cow<'_, str> => str);
forward_encode_deref!(Box<str> => str);
forward_encode_deref!(String => str);

impl sqlx::Decode<'_, Durable> for String {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(text) = value.0.as_text() {
            return Ok(text);
        }

        Err(unexpected_nonnull_type(&TypeInfo::text(), value))
    }
}

impl<'r> sqlx::Decode<'r, Durable> for Cow<'_, str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <String as sqlx::Decode<Durable>>::decode(value).map(Cow::Owned)
    }
}

impl sqlx::Decode<'_, Durable> for Box<str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <String as sqlx::Decode<Durable>>::decode(value).map(|s| s.into_boxed_str())
    }
}

impl sqlx::Type<Durable> for String {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::text()
    }
}

forward_type!(&'_ str => String);
forward_type!(Box<str> => String);
forward_type!(Cow<'_, str> => String);

impl sqlx::Encode<'_, Durable> for [&str] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::text_array(self)));
        Ok(IsNull::No)
    }
}

forward_slice_encode_deref!(&'_ str);

impl sqlx::Encode<'_, Durable> for [String] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let values = self.iter().map(|s| s.as_str()).collect();
        <Vec<&str> as sqlx::Encode<Durable>>::encode(values, buf)
    }
}

forward_slice_encode_deref!(String);

impl sqlx::Decode<'_, Durable> for Vec<String> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_text_array() {
            return Ok(value);
        }

        Err(unexpected_nonnull_type(&TypeInfo::text_array(), value))
    }
}

impl sqlx::Type<Durable> for Vec<String> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::text_array()
    }
}

forward_type!(Vec<&str> => Vec<String>);
forward_slice_type!(&str);
forward_slice_type!(String);
