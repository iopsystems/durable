use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::{encode_by_ref, unexpected_nonnull_type};
use crate::driver::{TypeInfo, Value};
use crate::{bindings as sql, Durable};

impl sqlx::Encode<'_, Durable> for [u8] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value::new(sql::Value::bytea(self)));
        Ok(IsNull::No)
    }
}

forward_encode_deref!(&'_ [u8] => [u8]);
forward_encode_deref!(Vec<u8> => [u8]);
forward_encode_deref!(Box<[u8]> => [u8]);
forward_encode_deref!(Cow<'_, [u8]> => [u8]);

impl<const N: usize> sqlx::Encode<'_, Durable> for [u8; N] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        encode_by_ref::<[u8]>(self, buf)
    }
}

impl sqlx::Type<Durable> for Vec<u8> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::bytea()
    }
}

forward_type!([u8] => Vec<u8>);
forward_type!(Box<[u8]> => Vec<u8>);
forward_type!(Cow<'_, [u8]> => Vec<u8>);

impl<const N: usize> sqlx::Type<Durable> for [u8; N] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        <Vec<u8> as sqlx::Type<Durable>>::type_info()
    }
}

impl sqlx::Encode<'_, Durable> for [&[u8]] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::bytea_array(self)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for [Vec<u8>] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let vec: Vec<_> = self.iter().map(|x| &x[..]).collect();
        encode_by_ref::<[&[u8]]>(&vec, buf)
    }
}

forward_slice_encode_deref!(&'_ [u8]);
forward_slice_encode_deref!(Vec<u8>);

impl sqlx::Decode<'_, Durable> for Vec<Vec<u8>> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        match value.0.as_bytea_array() {
            Some(value) => Ok(value),
            None => Err(unexpected_nonnull_type(&TypeInfo::bytea_array(), value)),
        }
    }
}

impl sqlx::Type<Durable> for Vec<Vec<u8>> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::bytea_array()
    }
}

forward_type!(Vec<&'_ [u8]> => Vec<Vec<u8>>);
forward_slice_type!(&'_ [u8]);
forward_slice_type!(Vec<u8>);
