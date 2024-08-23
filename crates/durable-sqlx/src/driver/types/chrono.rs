use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset, TimeZone, Utc};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use super::unexpected_nonnull_type;
use crate::bindings::durable::core::sql;
use crate::driver::{Durable, TypeInfo, Value};

impl<Tz: TimeZone> sqlx::Encode<'_, Durable> for DateTime<Tz> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::timestamptz(self.into())));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for DateTime<FixedOffset> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(ts) = value.0.as_timestamptz() {
            return Ok(ts.into());
        }

        Err(unexpected_nonnull_type("timestamptz", value))
    }
}

impl sqlx::Decode<'_, Durable> for DateTime<Utc> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <DateTime<FixedOffset> as sqlx::Decode<Durable>>::decode(value).map(|ts| ts.to_utc())
    }
}

impl sqlx::Decode<'_, Durable> for DateTime<Local> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        <DateTime<Utc> as sqlx::Decode<Durable>>::decode(value)
            .map(|ts| Local.from_utc_datetime(&ts.naive_utc()))
    }
}

impl<Tz: TimeZone> sqlx::Type<Durable> for DateTime<Tz> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamptz()
    }
}

impl sqlx::Encode<'_, Durable> for NaiveDateTime {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(Value::new(sql::Value::timestamp((*self).into())));
        Ok(IsNull::No)
    }
}

impl sqlx::Decode<'_, Durable> for NaiveDateTime {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(ts) = value.0.as_timestamp() {
            return Ok(ts.into());
        }

        Err(unexpected_nonnull_type("timestamp", value))
    }
}

impl sqlx::Type<Durable> for NaiveDateTime {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamp()
    }
}

impl<Tz: TimeZone> sqlx::Encode<'_, Durable> for &'_ [DateTime<Tz>] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let values: Vec<_> = self.iter().map(|ts| ts.into()).collect();
        buf.push(Value::new(sql::Value::timestamptz_array(&values)));
        Ok(IsNull::No)
    }
}

impl<Tz: TimeZone> sqlx::Encode<'_, Durable> for Vec<DateTime<Tz>> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[DateTime<Tz>] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<DateTime<FixedOffset>> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(values) = value.0.as_timestamptz_array() {
            let values = values.into_iter().map(From::from).collect();

            return Ok(values);
        }

        Err(unexpected_nonnull_type("timestamptz[]", value))
    }
}

impl sqlx::Decode<'_, Durable> for Vec<DateTime<Utc>> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(values) = value.0.as_timestamptz_array() {
            let values = values
                .into_iter()
                .map(|ts| DateTime::from(ts).to_utc())
                .collect();

            return Ok(values);
        }

        Err(unexpected_nonnull_type("timestamptz[]", value))
    }
}

impl sqlx::Decode<'_, Durable> for Vec<DateTime<Local>> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(values) = value.0.as_timestamptz_array() {
            let values = values
                .into_iter()
                .map(|ts| Local.from_utc_datetime(&DateTime::from(ts).naive_utc()))
                .collect();

            return Ok(values);
        }

        Err(unexpected_nonnull_type("timestamptz[]", value))
    }
}

impl<Tz: TimeZone> sqlx::Type<Durable> for &'_ [DateTime<Tz>] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamptz_array()
    }
}

impl<Tz: TimeZone> sqlx::Type<Durable> for Vec<DateTime<Tz>> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamptz_array()
    }
}

impl sqlx::Encode<'_, Durable> for &'_ [NaiveDateTime] {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        let values: Vec<_> = self.iter().copied().map(|ts| ts.into()).collect();
        buf.push(Value::new(sql::Value::timestamp_array(&values)));
        Ok(IsNull::No)
    }
}

impl sqlx::Encode<'_, Durable> for Vec<NaiveDateTime> {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&[NaiveDateTime] as sqlx::Encode<Durable>>::encode(self, buf)
    }
}

impl sqlx::Decode<'_, Durable> for Vec<NaiveDateTime> {
    fn decode(value: <Durable as sqlx::Database>::ValueRef<'_>) -> Result<Self, BoxDynError> {
        if let Some(values) = value.0.as_timestamp_array() {
            let values = values.into_iter().map(From::from).collect();

            return Ok(values);
        }

        Err(unexpected_nonnull_type("timestamp[]", value))
    }
}

impl sqlx::Type<Durable> for &'_ [NaiveDateTime] {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamp_array()
    }
}

impl sqlx::Type<Durable> for Vec<NaiveDateTime> {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::timestamp_array()
    }
}

impl From<sql::Timestamp> for NaiveDateTime {
    fn from(value: sql::Timestamp) -> Self {
        #[allow(deprecated)]
        NaiveDateTime::from_timestamp(value.seconds, value.subsec_nanos)
    }
}

impl From<NaiveDateTime> for sql::Timestamp {
    fn from(ts: NaiveDateTime) -> Self {
        #[allow(deprecated)]
        sql::Timestamp {
            seconds: ts.timestamp(),
            subsec_nanos: ts.timestamp_subsec_nanos(),
        }
    }
}

impl From<sql::Timestamptz> for DateTime<FixedOffset> {
    fn from(timestamp: sql::Timestamptz) -> Self {
        DateTime::from_naive_utc_and_offset(
            sql::Timestamp {
                seconds: timestamp.seconds,
                subsec_nanos: timestamp.subsec_nanos,
            }
            .into(),
            FixedOffset::west_opt(timestamp.offset).expect("timestamp offset was out of range"),
        )
    }
}

impl<Tz: TimeZone> From<&'_ DateTime<Tz>> for sql::Timestamptz {
    fn from(ts: &'_ DateTime<Tz>) -> Self {
        let offset = ts.offset().fix();

        sql::Timestamptz {
            seconds: ts.timestamp(),
            subsec_nanos: ts.timestamp_subsec_nanos(),
            offset: offset.local_minus_utc(),
        }
    }
}

impl<Tz: TimeZone> From<DateTime<Tz>> for sql::Timestamptz {
    fn from(value: DateTime<Tz>) -> Self {
        Self::from(&value)
    }
}
