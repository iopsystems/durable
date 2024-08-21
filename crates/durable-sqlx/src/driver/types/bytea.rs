use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, Durable};

impl<'r> sqlx::Decode<'r, Durable> for Cow<'_, [u8]> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <Vec<u8> as sqlx::Decode<Durable>>::decode(value).map(Cow::Owned)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<u8> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.0.as_bytea() {
            Some(bytes) => Ok(bytes),
            None => Err(unexpected_nonnull_type("bytea", value)),
        }
    }
}

impl sqlx::Encode<'_, Durable> for &[u8] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::bytea(self)));
        Ok(IsNull::No)
    }
}

impl<const N: usize> sqlx::Encode<'_, Durable> for [u8; N] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Box<[u8]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<u8> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Type<Durable> for Vec<u8> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::bytea()
    }
}

impl sqlx::Type<Durable> for Cow<'_, [u8]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Type<Durable> for Box<[u8]> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<Durable>>::type_info()
    }
}

impl<const N: usize> sqlx::Type<Durable> for [u8; N] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Encode<'_, Durable> for Vec<Vec<u8>> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::bytea_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for &'_ [Vec<u8>] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::bytea_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<Vec<u8>> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.0.as_bytea_array() {
            Some(value) => Ok(value),
            None => Err(unexpected_nonnull_type("bytea[]", value)),
        }
    }
}

impl sqlx::Type<Durable> for Vec<Vec<u8>> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::bytea_array()
    }
}
