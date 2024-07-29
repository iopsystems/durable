use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::Encode;

use crate::bindings as sql;
use crate::driver::{Durable, Value};

impl Encode<'_, Durable> for &[u8] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Bytea(self.to_vec())));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for &str {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Text(self.to_string())));
        Ok(IsNull::No)
    }
}

impl<const N: usize> Encode<'_, Durable> for [u8; N] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as Encode<Durable>>::encode(self, buf)
    }
}

impl Encode<'_, Durable> for Box<[u8]> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as Encode<Durable>>::encode(self, buf)
    }
}

impl Encode<'_, Durable> for Box<str> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<Durable>>::encode(self, buf)
    }
}

impl Encode<'_, Durable> for Cow<'_, str> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<Durable>>::encode(self, buf)
    }
}

impl Encode<'_, Durable> for String {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&str as Encode<Durable>>::encode(self, buf)
    }

    fn encode(
        self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Text(self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for Vec<u8> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        <&[u8] as Encode<Durable>>::encode(self, buf)
    }

    fn encode(
        self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Bytea(self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Boolean(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for f32 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Float4(*self)));
        Ok(IsNull::No)
    }
}

impl Encode<'_, Durable> for f64 {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        buf.push(Value(sql::Value::Float8(*self)));
        Ok(IsNull::No)
    }
}
