use async_stream::try_stream;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures_util::{StreamExt, TryStreamExt};
use serde_json::value::RawValue;
use sqlx::postgres::PgTypeInfo;
use sqlx::types::chrono::FixedOffset;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::types::Json;
use sqlx::{Column, Row};
use uuid::Uuid;
use value::ValueResource;
use wasmtime::component::Resource;

use self::type_info::TypeInfoResource;
use self::value::Value;
use crate::bindings::durable::core::sql::{self, Host};
use crate::resource::Resourceable;
use crate::task::QueryResult;
use crate::Task;

mod oids;
mod type_info;
mod value;

impl Resourceable for sql::TypeInfo {
    const NAME: &'static str = "durable:core/sql.type-info";

    type Data = TypeInfoResource;
}

impl Resourceable for sql::Value {
    const NAME: &'static str = "durable:core/sql.value";

    type Data = ValueResource;
}

fn type_info<'q, T>(value: &T) -> TypeInfoResource
where
    T: sqlx::Encode<'q, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    TypeInfoResource {
        type_info: value.produces().unwrap_or_else(T::type_info),
        name: None,
    }
}

impl sql::HostTypeInfo for Task {
    async fn name(&mut self, res: Resource<sql::TypeInfo>) -> wasmtime::Result<String> {
        use sqlx::TypeInfo;

        let tyinfo = self.resources.get(res)?;
        Ok(tyinfo.name().to_owned())
    }

    async fn compatible(
        &mut self,
        a: Resource<sql::TypeInfo>,
        b: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<bool> {
        use sqlx::TypeInfo;

        let tya = self.resources.get(a)?;
        let tyb = self.resources.get(b)?;

        Ok(tya.type_compatible(tyb))
    }

    async fn equal(
        &mut self,
        a: Resource<sql::TypeInfo>,
        b: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<bool> {
        let tya = self.resources.get(a)?;
        let tyb = self.resources.get(b)?;

        Ok(tya.type_eq(tyb))
    }

    async fn clone(
        &mut self,
        res: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        let tyinfo = self.resources.get(res)?.clone();
        self.resources.insert(tyinfo)
    }

    async fn serialize(
        &mut self,
        res: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<Result<String, String>> {
        let tyinfo = self.resources.get(res)?;
        let json = serde_json::to_string(tyinfo).map_err(|e| e.to_string());

        Ok(json)
    }

    async fn deserialize(
        &mut self,
        json: String,
    ) -> wasmtime::Result<Result<Resource<sql::TypeInfo>, String>> {
        let tyinfo: TypeInfoResource = match serde_json::from_str(&json) {
            Ok(tyinfo) => tyinfo,
            Err(e) => return Ok(Err(e.to_string())),
        };

        Ok(Ok(self.resources.insert(tyinfo)?))
    }

    async fn with_name(
        &mut self,
        name: String,
    ) -> wasmtime::Result<Result<Resource<sql::TypeInfo>, String>> {
        let pool = self.state.pool();

        let result = sqlx::query_scalar!(r#"SELECT $1::regtype::oid as "oid!""#, &name as &str)
            .fetch_one(pool)
            .await;

        let oid = match result {
            Ok(oid) => oid,
            Err(sqlx::Error::Database(err)) if err.code().as_deref() == Some("42704") => {
                return Ok(Err(err.to_string()))
            }
            Err(e) => return Err(e.into()),
        };

        let tyinfo = PgTypeInfo::with_oid(oid);

        Ok(Ok(self.resources.insert(TypeInfoResource {
            type_info: tyinfo,
            name: Some(name),
        })?))
    }

    async fn boolean(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<bool as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn float4(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<f32 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn float8(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<f64 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int1(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<i8 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int2(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<i16 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int4(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<i32 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int8(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<i64 as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn text(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<String as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn bytea(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<u8> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn timestamptz(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<DateTime<Utc> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn timestamp(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<NaiveDateTime as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn uuid(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Uuid as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn jsonb(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Json<()> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn inet(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<IpNetwork as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn boolean_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<bool> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn float4_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<f32> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn float8_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<f64> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int1_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<i8> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int2_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<i16> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int4_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<i32> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn int8_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<i64> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn text_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<String> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn bytea_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<Vec<u8>> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn timestamptz_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<DateTime<Utc>> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn timestamp_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<NaiveDateTime> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn uuid_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<Uuid> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn jsonb_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<Json<()>> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn inet_array(&mut self) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        self.resources
            .insert(<Vec<IpNetwork> as sqlx::Type<sqlx::Postgres>>::type_info().into())
    }

    async fn drop(&mut self, rep: Resource<sql::TypeInfo>) -> wasmtime::Result<()> {
        self.resources.remove(rep)?;
        Ok(())
    }
}

impl sql::HostValue for Task {
    async fn is_null(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<bool> {
        let value = self.resources.get(res)?;
        Ok(value.value.is_null())
    }

    async fn type_info(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Resource<sql::TypeInfo>> {
        let value = self.resources.get(res)?;
        let tyinfo = value.type_info.clone();
        self.resources.insert(tyinfo)
    }

    async fn clone(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = self.resources.get(res)?.clone();
        self.resources.insert(value)
    }

    async fn serialize(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Result<String, String>> {
        let tyinfo = self.resources.get(res)?;
        let json = serde_json::to_string(tyinfo).map_err(|e| e.to_string());

        Ok(json)
    }

    async fn deserialize(
        &mut self,
        json: String,
    ) -> wasmtime::Result<Result<Resource<sql::Value>, String>> {
        let value: ValueResource = match serde_json::from_str(&json) {
            Ok(tyinfo) => tyinfo,
            Err(e) => return Ok(Err(e.to_string())),
        };

        Ok(Ok(self.resources.insert(value)?))
    }

    async fn as_boolean(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<bool>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Boolean(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_float4(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<f32>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Float4(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_float8(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<f64>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Float8(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_int1(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<i8>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int1(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_int2(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<i16>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int2(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_int4(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<i32>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int4(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_int8(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<i64>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int8(v) => Some(*v),
            _ => None,
        })
    }

    async fn as_text(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<String>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Text(v) => Some(v.clone()),
            _ => None,
        })
    }

    async fn as_bytea(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<Vec<u8>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Bytea(v) => Some(v.clone()),
            _ => None,
        })
    }

    async fn as_timestamptz(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<sql::Timestamptz>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            &Value::TimestampTz(ts) => Some(ts.into()),
            _ => None,
        })
    }

    async fn as_timestamp(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<sql::Timestamp>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            &Value::Timestamp(ts) => Some(ts.into()),
            _ => None,
        })
    }

    async fn as_uuid(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<sql::Uuid>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            &Value::Uuid(uuid) => Some(uuid.into()),
            _ => None,
        })
    }

    async fn as_json(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<Option<String>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Json(json) => Some(json.get().to_owned()),
            _ => None,
        })
    }

    async fn as_inet(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<sql::IpNetwork>> {
        let value = self.resources.get(res)?;

        Ok(match value.value {
            Value::Inet(inet) => Some(inet.into()),
            _ => None,
        })
    }

    async fn as_boolean_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<bool>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::BooleanArray(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_float4_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<f32>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Float4Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_float8_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<f64>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Float8Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_int1_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<i8>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int1Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_int2_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<i16>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int2Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_int4_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<i32>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int4Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_int8_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<i64>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::Int8Array(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_text_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<String>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::TextArray(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_bytea_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<Vec<u8>>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::ByteaArray(arr) => Some(arr.clone()),
            _ => None,
        })
    }

    async fn as_timestamptz_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<sql::Timestamptz>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::TimestampTzArray(arr) => Some(arr.iter().copied().map(From::from).collect()),
            _ => None,
        })
    }

    async fn as_timestamp_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<sql::Timestamp>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::TimestampArray(arr) => Some(arr.iter().copied().map(From::from).collect()),
            _ => None,
        })
    }

    async fn as_uuid_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<sql::Uuid>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::UuidArray(arr) => Some(arr.iter().copied().map(From::from).collect()),
            _ => None,
        })
    }

    async fn as_json_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<String>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::JsonArray(arr) => Some(arr.iter().map(|json| json.get().to_owned()).collect()),
            _ => None,
        })
    }

    async fn as_inet_array(
        &mut self,
        res: Resource<sql::Value>,
    ) -> wasmtime::Result<Option<Vec<sql::IpNetwork>>> {
        let value = self.resources.get(res)?;

        Ok(match &value.value {
            Value::InetArray(arr) => Some(arr.iter().copied().map(From::from).collect()),
            _ => None,
        })
    }

    async fn null(
        &mut self,
        tyinfo: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let tyinfo = self.resources.remove(tyinfo)?;
        let value = ValueResource {
            type_info: tyinfo,
            value: Value::Null,
        };

        self.resources.insert(value)
    }

    async fn boolean(&mut self, value: bool) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Boolean(value),
        };

        self.resources.insert(value)
    }

    async fn float4(&mut self, value: f32) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Float4(value),
        };

        self.resources.insert(value)
    }

    async fn float8(&mut self, value: f64) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Float8(value),
        };

        self.resources.insert(value)
    }

    async fn int1(&mut self, value: i8) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int1(value),
        };

        self.resources.insert(value)
    }

    async fn int2(&mut self, value: i16) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int2(value),
        };

        self.resources.insert(value)
    }

    async fn int4(&mut self, value: i32) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int4(value),
        };

        self.resources.insert(value)
    }

    async fn int8(&mut self, value: i64) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int8(value),
        };

        self.resources.insert(value)
    }

    async fn text(&mut self, value: String) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Text(value),
        };

        self.resources.insert(value)
    }

    async fn bytea(&mut self, value: Vec<u8>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Bytea(value),
        };

        self.resources.insert(value)
    }

    async fn timestamptz(
        &mut self,
        value: sql::Timestamptz,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let value: DateTime<FixedOffset> = value.into();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::TimestampTz(value),
        };

        self.resources.insert(value)
    }

    async fn timestamp(&mut self, value: sql::Timestamp) -> wasmtime::Result<Resource<sql::Value>> {
        let value: NaiveDateTime = value.into();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Timestamp(value),
        };

        self.resources.insert(value)
    }

    async fn uuid(&mut self, value: sql::Uuid) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value.into();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Uuid(value),
        };

        self.resources.insert(value)
    }

    async fn jsonb(&mut self, value: String) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value.into_boxed_str();
        let value: Box<RawValue> = unsafe { std::mem::transmute(value) };
        let value = ValueResource {
            type_info: type_info(&Json(&value)),
            value: Value::Json(Json(value)),
        };

        self.resources.insert(value)
    }

    async fn inet(
        &mut self,
        value: sql::IpNetwork,
    ) -> wasmtime::Result<Result<Resource<sql::Value>, String>> {
        let value: IpNetwork = match value.try_into() {
            Ok(value) => value,
            Err(e) => return Ok(Err(e.to_string())),
        };
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Inet(value),
        };

        self.resources.insert(value).map(Ok)
    }

    async fn enum_value(
        &mut self,
        value: String,
        tyinfo: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let tyinfo = self.resources.get(tyinfo)?.clone();
        let value = ValueResource {
            type_info: tyinfo,
            value: Value::Text(value),
        };

        self.resources.insert(value)
    }

    async fn boolean_array(&mut self, value: Vec<bool>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::BooleanArray(value),
        };

        self.resources.insert(value)
    }

    async fn float4_array(&mut self, value: Vec<f32>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Float4Array(value),
        };

        self.resources.insert(value)
    }

    async fn float8_array(&mut self, value: Vec<f64>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Float8Array(value),
        };

        self.resources.insert(value)
    }

    async fn int1_array(&mut self, value: Vec<i8>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int1Array(value),
        };

        self.resources.insert(value)
    }

    async fn int2_array(&mut self, value: Vec<i16>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int2Array(value),
        };

        self.resources.insert(value)
    }

    async fn int4_array(&mut self, value: Vec<i32>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int4Array(value),
        };

        self.resources.insert(value)
    }

    async fn int8_array(&mut self, value: Vec<i64>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::Int8Array(value),
        };

        self.resources.insert(value)
    }

    async fn text_array(&mut self, value: Vec<String>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::TextArray(value),
        };

        self.resources.insert(value)
    }

    async fn bytea_array(&mut self, value: Vec<Vec<u8>>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::ByteaArray(value),
        };

        self.resources.insert(value)
    }

    async fn timestamptz_array(
        &mut self,
        value: Vec<sql::Timestamptz>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value.into_iter().map(From::from).collect();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::TimestampTzArray(value),
        };

        self.resources.insert(value)
    }

    async fn timestamp_array(
        &mut self,
        value: Vec<sql::Timestamp>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value.into_iter().map(From::from).collect();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::TimestampArray(value),
        };

        self.resources.insert(value)
    }

    async fn uuid_array(
        &mut self,
        value: Vec<sql::Uuid>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value.into_iter().map(From::from).collect();
        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::UuidArray(value),
        };

        self.resources.insert(value)
    }

    async fn jsonb_array(&mut self, value: Vec<String>) -> wasmtime::Result<Resource<sql::Value>> {
        let value = value
            .into_iter()
            .map(|json| {
                let json = json.into_boxed_str();
                let json: Box<RawValue> = unsafe { std::mem::transmute(json) };
                Json(json)
            })
            .collect();

        let value = ValueResource {
            type_info: type_info(&value),
            value: Value::JsonArray(value),
        };

        self.resources.insert(value)
    }

    async fn inet_array(
        &mut self,
        values: Vec<sql::IpNetwork>,
    ) -> wasmtime::Result<Result<Resource<sql::Value>, String>> {
        let values: Vec<IpNetwork> = match values.into_iter().map(TryFrom::try_from).collect() {
            Ok(values) => values,
            Err(e) => return Ok(Err(e.to_string())),
        };
        let value = ValueResource {
            type_info: type_info(&values),
            value: Value::InetArray(values),
        };

        self.resources.insert(value).map(Ok)
    }

    async fn enum_array(
        &mut self,
        value: Vec<String>,
        tyinfo: Resource<sql::TypeInfo>,
    ) -> wasmtime::Result<Resource<sql::Value>> {
        let tyinfo = self.resources.get(tyinfo)?.clone();
        let value = ValueResource {
            type_info: tyinfo,
            value: Value::TextArray(value),
        };

        self.resources.insert(value)
    }

    async fn drop(&mut self, res: Resource<sql::Value>) -> wasmtime::Result<()> {
        self.resources.remove(res)?;
        Ok(())
    }
}

impl Host for Task {
    async fn query(
        &mut self,
        sql: String,
        param_res: Vec<Resource<sql::Value>>,
        options: sql::Options,
    ) -> anyhow::Result<()> {
        let txn = self.state.assert_in_transaction("durable::sql::query")?;

        let mut params = Vec::with_capacity(param_res.len());
        for param in param_res {
            params.push(self.resources.remove(param)?);
        }

        txn.start_query(move |conn| {
            Box::pin(try_stream! {
                let mut query = sqlx::query(&sql).persistent(options.persistent);
                for param in params {
                    query = query.bind(param);
                }

                match options.limit {
                    0 => {
                        let result = query.execute(&mut **conn).await?;
                        yield sqlx::Either::Left(QueryResult::from(result));
                    },
                    1 =>  {
                        let row = query.fetch_optional(&mut **conn).await?;

                        let count = match &row {
                            Some(_) => 1,
                            None => 0
                        };

                        yield sqlx::Either::Left(QueryResult {
                            rows_affected: count
                        });

                        if let Some(row) = row {
                            yield sqlx::Either::Right(row);
                        }
                    },
                    _ => {
                        // We don't allow multiple statements at once but this is the only way to
                        // recover both the query result and all the query rows.
                        #[allow(deprecated)]
                        let result = query.fetch_many(&mut **conn);


                        let mut result = result;
                        while let Some(item) = result.try_next().await? {
                            yield item.map_left(QueryResult::from);
                        }
                    }
                }
            })
        })
    }

    async fn fetch(&mut self) -> wasmtime::Result<Option<Result<sql::QueryResult, sql::Error>>> {
        let txn = self.state.assert_in_transaction("durable::sql::query")?;
        let stream = match txn.stream() {
            Some(stream) => stream,
            None => return Ok(None),
        };

        let item = match stream.next().await {
            Some(item) => item,
            None => return Ok(None),
        };

        Ok(Some(match item {
            Ok(sqlx::Either::Left(result)) => Ok(sql::QueryResult::Count(result.rows_affected)),
            Ok(sqlx::Either::Right(row)) => {
                let mut columns = Vec::with_capacity(row.len());

                for (idx, column) in row.columns().iter().enumerate() {
                    let value: ValueResource = match row.try_get(idx) {
                        Ok(value) => value,
                        Err(e) => return Ok(Some(Err(convert_sqlx_error(e)?))),
                    };

                    let value = self.resources.insert(value)?;

                    columns.push(sql::Column {
                        name: column.name().to_owned(),
                        value,
                    });
                }

                Ok(sql::QueryResult::Row(sql::Row { columns }))
            }
            Err(e) => Err(convert_sqlx_error(e)?),
        }))
    }
}

fn convert_sqlx_error(err: sqlx::Error) -> anyhow::Result<sql::Error> {
    use sqlx::error::ErrorKind;

    // Most errors basically convert directly to sql::Error::Other because they are
    // either extremely unlikely or not something the workflow program can usefully
    // do something with.
    Ok(match err {
        sqlx::Error::ColumnDecode { index, source } => {
            sql::Error::ColumnDecode(sql::ColumnDecodeError {
                index,
                source: source.to_string(),
            })
        }

        sqlx::Error::Database(e) => sql::Error::Database(sql::DatabaseError {
            message: e.message().to_owned(),
            code: e.code().map(|code| code.into_owned()),
            kind: match e.kind() {
                ErrorKind::UniqueViolation => sql::DatabaseErrorKind::UniqueViolation,
                ErrorKind::ForeignKeyViolation => sql::DatabaseErrorKind::ForeignKeyViolation,
                ErrorKind::NotNullViolation => sql::DatabaseErrorKind::NotNullViolation,
                ErrorKind::CheckViolation => sql::DatabaseErrorKind::CheckViolation,
                _ => sql::DatabaseErrorKind::Other,
            },
            constraint: e.constraint().map(|e| e.to_owned()),
            table: e.table().map(|e| e.to_owned()),
        }),
        sqlx::Error::Encode(e) => sql::Error::Encode(e.to_string()),
        sqlx::Error::Decode(e) => sql::Error::Decode(e.to_string()),
        sqlx::Error::TypeNotFound { type_name } => sql::Error::TypeNotFound(type_name),

        // These errors _shouldn't_ happen but should be passed on up if they do anyway.
        e @ sqlx::Error::Configuration(_) => sql::Error::Other(e.to_string()),
        e @ sqlx::Error::RowNotFound => sql::Error::Other(e.to_string()),

        // These errors should be impossible or otherwise indicate a bug in our code.
        // =============
        // This should never happen because we are just iterating over the list of columns that we
        // got from postgresql.
        e @ sqlx::Error::ColumnIndexOutOfBounds { .. } => {
            unreachable!("sqlx driver requested a column that was out of bounds: {e}")
        }
        // This should never happen because we don't request columns by name. That happens within
        // the guest program.
        e @ sqlx::Error::ColumnNotFound(_) => {
            unreachable!("sqlx driver failed to get a column by name: {e}")
        }

        // Fallback: The error is not one that should be passed on up to the worker so we emit a
        // trap.
        e => return Err(anyhow::Error::new(e)),
    })
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

impl From<DateTime<FixedOffset>> for sql::Timestamptz {
    fn from(ts: DateTime<FixedOffset>) -> Self {
        let offset = ts.timezone();

        sql::Timestamptz {
            seconds: ts.timestamp(),
            subsec_nanos: ts.timestamp_subsec_nanos(),
            offset: offset.local_minus_utc(),
        }
    }
}

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

impl From<IpNetwork> for sql::IpNetwork {
    fn from(value: IpNetwork) -> Self {
        match value {
            IpNetwork::V4(v4) => sql::IpNetwork::V4(sql::Ipv4Network {
                addr: v4.ip().to_bits(),
                prefix: v4.prefix(),
            }),
            IpNetwork::V6(v6) => {
                let bits = v6.ip().to_bits();
                let lo = bits as u64;
                let hi = (bits >> 64) as u64;

                sql::IpNetwork::V6(sql::Ipv6Network {
                    addr: (lo, hi),
                    prefix: v6.prefix(),
                })
            }
        }
    }
}

impl TryFrom<sql::IpNetwork> for IpNetwork {
    type Error = ipnetwork::IpNetworkError;

    fn try_from(value: sql::IpNetwork) -> Result<Self, Self::Error> {
        use std::net::{Ipv4Addr, Ipv6Addr};

        use ipnetwork::{Ipv4Network, Ipv6Network};

        Ok(match value {
            sql::IpNetwork::V4(v4) => {
                Self::V4(Ipv4Network::new(Ipv4Addr::from_bits(v4.addr), v4.prefix)?)
            }
            sql::IpNetwork::V6(v6) => {
                let addr = (v6.addr.0 as u128) | (v6.addr.1 as u128) << 64;

                Self::V6(Ipv6Network::new(Ipv6Addr::from_bits(addr), v6.prefix)?)
            }
        })
    }
}
