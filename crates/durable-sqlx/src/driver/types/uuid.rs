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
        let uuid: [u8; 16] = self.to_bytes_le();
        let uuid: sql::Uuid = unsafe { std::mem::transmute(uuid) };

        buf.push(Value::new(sql::Value::uuid(uuid)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for Uuid {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_uuid() {
            let uuid: [u8; 16] = unsafe { std::mem::transmute(value) };

            return Ok(Uuid::from_bytes_le(uuid));
        }

        Err(unexpected_nonnull_type("uuid", value))
    }
}

impl sqlx::Type<Durable> for Uuid {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::uuid()
    }
}

impl sqlx::Encode<'_, Durable> for &'_ [Uuid] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let array: Vec<sql::Uuid> = self
            .iter()
            .map(|uuid| {
                let uuid: [u8; 16] = uuid.to_bytes_le();
                let uuid: sql::Uuid = unsafe { std::mem::transmute(uuid) };
                uuid
            })
            .collect();

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
        let array: Vec<sql::Uuid> = self
            .into_iter()
            .map(|uuid| {
                let uuid: [u8; 16] = uuid.to_bytes_le();
                let uuid: sql::Uuid = unsafe { std::mem::transmute(uuid) };
                uuid
            })
            .collect();

        buf.push(Value(sql::Value::uuid_array(&array)));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<Uuid> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(value) = value.0.as_uuid_array() {
            let array: Vec<Uuid> = value
                .into_iter()
                .map(|uuid| {
                    let uuid: [u8; 16] = unsafe { std::mem::transmute(uuid) };
                    Uuid::from_bytes_le(uuid)
                })
                .collect();

            return Ok(array);
        }

        Err(unexpected_nonnull_type("uuid[]", value))
    }
}

impl sqlx::Type<Durable> for &'_ [Uuid] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::uuid_array()
    }
}

impl sqlx::Type<Durable> for Vec<Uuid> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::uuid_array()
    }
}
