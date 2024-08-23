use std::borrow::Cow;

use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use uuid::Uuid;

use super::unexpected_nonnull_type;
use crate::bindings::durable::core::sql;
use crate::driver::{Durable, TypeInfo, Value};

impl sqlx::Encode<'_, Durable> for Uuid {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::uuid((*self).into())));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for Uuid {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_uuid() {
            return Ok(value.into());
        }

        Err(unexpected_nonnull_type(&TypeInfo::uuid(), value))
    }
}

impl sqlx::Type<Durable> for Uuid {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::uuid()
    }
}

impl sqlx::Encode<'_, Durable> for [Uuid] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let array: Vec<sql::Uuid> = self.iter().copied().map(From::from).collect();

        buf.push(Value(sql::Value::uuid_array(&array)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<Uuid> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[Uuid] as sqlx::Encode<'_, Durable>>::encode(self, buf)
    }

    fn encode(
        self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let array: Vec<sql::Uuid> = self.into_iter().map(From::from).collect();

        buf.push(Value(sql::Value::uuid_array(&array)));
        Ok(IsNull::No)
    }
}

forward_encode_deref!(&'_ [Uuid] => [Uuid]);
forward_encode_deref!(Box<[Uuid]> => [Uuid]);
forward_encode_deref!(Cow<'_, [Uuid]> => [Uuid]);

impl sqlx::Decode<'_, Durable> for Vec<Uuid> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_uuid_array() {
            let array: Vec<Uuid> = value.into_iter().map(From::from).collect();

            return Ok(array);
        }

        Err(unexpected_nonnull_type(&TypeInfo::uuid_array(), value))
    }
}

impl sqlx::Type<Durable> for Vec<Uuid> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::uuid_array()
    }
}

forward_slice_type!(Uuid);

impl From<sql::Uuid> for Uuid {
    fn from(value: sql::Uuid) -> Self {
        Uuid::from_u64_pair(value.hi, value.lo)
    }
}

impl From<Uuid> for sql::Uuid {
    fn from(value: Uuid) -> Self {
        let (hi, lo) = value.as_u64_pair();
        sql::Uuid { hi, lo }
    }
}
