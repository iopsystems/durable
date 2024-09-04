use std::fmt;

use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde_json::value::RawValue;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::PgTypeInfo;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::types::Json;
use sqlx_postgres::PgTypeKind;
use uuid::Uuid;

use super::{oids as oid, TypeInfoResource};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ValueResource {
    pub(crate) type_info: TypeInfoResource,
    #[serde(flatten)]
    pub(crate) value: Value,
}

/// All supported postgres values.
///
/// New values can be added here without having to worry about backwards
/// compatibility. The WIT interface does not allow the client to exhaustively
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value")]
pub(crate) enum Value {
    Null,

    Boolean(bool),
    Float4(f32),
    Float8(f64),
    Int1(i8),
    Int2(i16),
    Int4(i32),
    Int8(i64),
    Text(String),
    Bytea(Vec<u8>),
    TimestampTz(DateTime<FixedOffset>),
    Timestamp(NaiveDateTime),
    Uuid(Uuid),
    Json(Json<Box<RawValue>>),
    Inet(IpNetwork),

    BooleanArray(Vec<bool>),
    Float4Array(Vec<f32>),
    Float8Array(Vec<f64>),
    Int1Array(Vec<i8>),
    Int2Array(Vec<i16>),
    Int4Array(Vec<i32>),
    Int8Array(Vec<i64>),
    TextArray(Vec<String>),
    ByteaArray(Vec<Vec<u8>>),
    TimestampTzArray(Vec<DateTime<FixedOffset>>),
    TimestampArray(Vec<NaiveDateTime>),
    UuidArray(Vec<Uuid>),
    JsonArray(Vec<Json<Box<RawValue>>>),
    InetArray(Vec<IpNetwork>),
}

macro_rules! for_each_value {
    {
        match $value:expr => {
            null => $nullexpr:expr,
            $var:ident => $result:expr,
        }
    } => {
        match $value {
            Value::Null => $nullexpr,
            Value::Boolean($var) => $result,
            Value::Float4($var) => $result,
            Value::Float8($var) => $result,
            Value::Int1($var) => $result,
            Value::Int2($var) => $result,
            Value::Int4($var) => $result,
            Value::Int8($var) => $result,
            Value::Text($var) => $result,
            Value::Bytea($var) => $result,
            Value::TimestampTz($var) => $result,
            Value::Timestamp($var) => $result,
            Value::Uuid($var) => $result,
            Value::Json($var) => $result,
            Value::Inet($var) => $result,

            Value::BooleanArray($var) => $result,
            Value::Float4Array($var) => $result,
            Value::Float8Array($var) => $result,
            Value::Int1Array($var) => $result,
            Value::Int2Array($var) => $result,
            Value::Int4Array($var) => $result,
            Value::Int8Array($var) => $result,
            Value::TextArray($var) => $result,
            Value::ByteaArray($var) => $result,
            Value::TimestampTzArray($var) => $result,
            Value::TimestampArray($var) => $result,
            Value::UuidArray($var) => $result,
            Value::JsonArray($var) => $result,
            Value::InetArray($var) => $result,
        }
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl sqlx::Type<sqlx::Postgres> for ValueResource {
    fn type_info() -> PgTypeInfo {
        // The real type is provided in Decode::provide
        PgTypeInfo::with_name("void")
    }

    fn compatible(_: &PgTypeInfo) -> bool {
        // We could check that types are compatible here but it is better to leave that
        // to the decode impl.
        true
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Postgres> for ValueResource {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, BoxDynError> {
        fn encode_by_ref<'a, T>(
            val: &T,
            buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
        ) -> Result<IsNull, BoxDynError>
        where
            T: sqlx::Encode<'a, sqlx::Postgres>,
        {
            val.encode_by_ref(buf)
        }

        for_each_value! {
            match &self.value => {
                null => Ok(IsNull::Yes),
                val => encode_by_ref(val, buf),
            }
        }
    }

    fn encode(
        self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, BoxDynError> {
        fn encode<'a, T>(
            val: T,
            buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
        ) -> Result<IsNull, BoxDynError>
        where
            T: sqlx::Encode<'a, sqlx::Postgres>,
        {
            val.encode(buf)
        }

        for_each_value! {
            match self.value => {
                null => Ok(IsNull::Yes),
                val => encode(val, buf),
            }
        }
    }

    fn produces(&self) -> Option<PgTypeInfo> {
        Some(self.type_info.type_info.clone())
    }

    fn size_hint(&self) -> usize {
        fn size_hint<'a, T>(val: &T) -> usize
        where
            T: sqlx::Encode<'a, sqlx::Postgres>,
        {
            val.size_hint()
        }

        for_each_value! {
            match &self.value => {
                null => 0,
                val => size_hint(val),
            }
        }
    }
}

impl<'a> sqlx::Decode<'a, sqlx::Postgres> for ValueResource {
    fn decode(
        value: <sqlx::Postgres as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, BoxDynError> {
        use sqlx::ValueRef;

        fn decode<'a, T>(
            value: <sqlx::Postgres as sqlx::Database>::ValueRef<'a>,
        ) -> Result<T, BoxDynError>
        where
            T: sqlx::Decode<'a, sqlx::Postgres>,
        {
            T::decode(value)
        }

        let type_info = value.type_info().into_owned();

        if value.is_null() {
            return Ok(Self {
                type_info: type_info.into(),
                value: Value::Null,
            });
        }

        let value = match &type_info {
            t if t.type_eq(&oid::BOOL) => decode(value).map(Value::Boolean),
            t if t.type_eq(&oid::FLOAT4) => decode(value).map(Value::Float4),
            t if t.type_eq(&oid::FLOAT8) => decode(value).map(Value::Float8),
            t if t.type_eq(&oid::CHAR) => decode(value).map(Value::Int1),
            t if t.type_eq(&oid::INT2) => decode(value).map(Value::Int2),
            t if t.type_eq(&oid::INT4) => decode(value).map(Value::Int4),
            t if t.type_eq(&oid::INT8) => decode(value).map(Value::Int8),
            t if t.type_eq(&oid::TEXT) => decode(value).map(Value::Text),
            t if t.type_eq(&oid::BYTEA) => decode(value).map(Value::Bytea),
            t if t.type_eq(&oid::TIMESTAMPTZ) => decode(value).map(Value::TimestampTz),
            t if t.type_eq(&oid::TIMESTAMP) => decode(value).map(Value::Timestamp),
            t if t.type_eq(&oid::UUID) => decode(value).map(Value::Uuid),
            t if t.type_eq(&oid::JSON) || t.type_eq(&oid::JSONB) => decode(value).map(Value::Json),
            t if t.type_eq(&oid::INET) => decode(value).map(Value::Inet),

            t if t.type_eq(&oid::BOOL_ARRAY) => decode(value).map(Value::BooleanArray),
            t if t.type_eq(&oid::FLOAT4_ARRAY) => decode(value).map(Value::Float4Array),
            t if t.type_eq(&oid::FLOAT8_ARRAY) => decode(value).map(Value::Float8Array),
            t if t.type_eq(&oid::CHAR_ARRAY) => decode(value).map(Value::Int1Array),
            t if t.type_eq(&oid::INT2_ARRAY) => decode(value).map(Value::Int2Array),
            t if t.type_eq(&oid::INT4_ARRAY) => decode(value).map(Value::Int4Array),
            t if t.type_eq(&oid::INT8_ARRAY) => decode(value).map(Value::Int8Array),
            t if t.type_eq(&oid::TEXT_ARRAY) => decode(value).map(Value::TextArray),
            t if t.type_eq(&oid::BYTEA_ARRAY) => decode(value).map(Value::ByteaArray),
            t if t.type_eq(&oid::TIMESTAMPTZ_ARRAY) => decode(value).map(Value::TimestampTzArray),
            t if t.type_eq(&oid::TIMESTAMP_ARRAY) => decode(value).map(Value::TimestampArray),
            t if t.type_eq(&oid::UUID_ARRAY) => decode(value).map(Value::UuidArray),
            t if t.type_eq(&oid::JSON_ARRAY) || t.type_eq(&oid::JSONB_ARRAY) => {
                decode(value).map(Value::JsonArray)
            }
            t if t.type_eq(&oid::INET_ARRAY) => decode(value).map(Value::InetArray),

            t if matches!(t.kind(), PgTypeKind::Enum(_)) => decode(value).map(Value::Text),

            _ => return Err(Box::new(UnsupportedType::new(type_info))),
        }?;

        Ok(Self {
            type_info: type_info.into(),
            value,
        })
    }
}

#[derive(Debug)]
pub struct UnsupportedType {
    type_info: PgTypeInfo,
}

impl UnsupportedType {
    pub fn new(type_info: PgTypeInfo) -> Self {
        Self { type_info }
    }
}

impl fmt::Display for UnsupportedType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use sqlx::TypeInfo;

        write!(f, "unsupported postgresql type `{}`", self.type_info.name())
    }
}

impl std::error::Error for UnsupportedType {}
