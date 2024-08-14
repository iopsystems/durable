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
        buf.push(Value(sql::Value::Text(self.to_string())));
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

    fn encode(
        self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Text(self)));
        Ok(IsNull::No)
    }
}

impl<'r> sqlx::Decode<'r, Durable> for &'r str {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        match &value.0 {
            sql::Value::Text(v) => Ok(&v),
            _ => Err(unexpected_nonnull_type("text", value)),
        }
    }
}

impl<'r> sqlx::Decode<'r, Durable> for Cow<'r, str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <&str as sqlx::Decode<Durable>>::decode(value).map(Cow::Borrowed)
    }
}

impl sqlx::Decode<'_, Durable> for Box<str> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <&str as sqlx::Decode<Durable>>::decode(value).map(|s| s.into())
    }
}

impl sqlx::Decode<'_, Durable> for String {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <&str as sqlx::Decode<Durable>>::decode(value).map(|v| v.to_owned())
    }
}

impl sqlx::Type<Durable> for String {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo(sql::PrimitiveType::Text)
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
