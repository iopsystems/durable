use std::fmt;

use async_stream::try_stream;
use futures_util::{StreamExt, TryStreamExt};
use sqlx::encode::IsNull;
use sqlx::postgres::types::Oid;
use sqlx::postgres::PgTypeInfo;
use sqlx::{Column, Row};

use crate::bindings::durable::core::sql::{self, Host};
use crate::task::QueryResult;
use crate::Task;

#[async_trait::async_trait]
impl Host for Task {
    async fn query(
        &mut self,
        sql: String,
        params: Vec<sql::Value>,
        options: sql::Options,
    ) -> anyhow::Result<()> {
        let txn = self.state.assert_in_transaction("durable::sql::query")?;

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

    async fn fetch(&mut self) -> anyhow::Result<Option<Result<sql::QueryResult, sql::Error>>> {
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
                    let value: sql::Value = match row.try_get(idx) {
                        Ok(value) => value,
                        Err(e) => return Ok(Some(Err(convert_sqlx_error(e)?))),
                    };

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

impl sqlx::Type<sqlx::Postgres> for sql::Value {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        // The real type is provided in Decode::provide
        PgTypeInfo::with_name("void")
    }

    fn compatible(_: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        true
    }
}

macro_rules! primitive_for_each {
    {
        match $self:expr => {
            $var:ident => $result:expr,
            null $ty:pat => $nexpr:expr $(,)?
        }
    } => {
        match $self {
            Self::Null($ty) => $nexpr,
            Self::Boolean($var) => $result,
            Self::Float4($var) => $result,
            Self::Float8($var) => $result,
            Self::Int1($var) => $result,
            Self::Int2($var) => $result,
            Self::Int4($var) => $result,
            Self::Int8($var) => $result,
            Self::Text($var) => $result,
            Self::Bytea($var) => $result,
        }
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Postgres> for sql::Value {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, sqlx::error::BoxDynError> {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&v, buf),
                null _ => Ok(IsNull::Yes)
            }
        }
    }

    fn encode(
        self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::encode(v, buf),
                null _ => Ok(IsNull::Yes)
            }
        }
    }

    fn produces(&self) -> Option<<sqlx::Postgres as sqlx::Database>::TypeInfo> {
        fn type_info<'a, T>(value: &'a T) -> PgTypeInfo 
        where
            T: sqlx::Encode<'a, sqlx::Postgres>,
            T: sqlx::Type<sqlx::Postgres>,
        {
            T::produces(value).unwrap_or_else(T::type_info)
        }

        primitive_for_each! {
            match &self => {
                v => Some(type_info(v)),
                null ty => Some(ty.into())
            }
        }
    }

    fn size_hint(&self) -> usize {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::size_hint(v),
                null _ => 0
            }
        }
    }
}

macro_rules! decl_oids {
    {
        $( const $name:ident = $value:expr; )*
    } => {
        $(
            #[allow(dead_code)]
            const $name: PgTypeInfo = PgTypeInfo::with_oid(Oid($value));
        )*
    }
}

// You can verify these values by running
//
// SELECT oid, typname FROM pg_type;
//
// Note that types with their name prefixed with `_` are array types.
decl_oids! {
    const PG_TYPE_BOOL = 16;
    const PG_TYPE_BYTEA = 17;
    const PG_TYPE_CHAR = 18;
    const PG_TYPE_NAME = 19;
    const PG_TYPE_INT8 = 20;
    const PG_TYPE_INT2 = 21;
    const PG_TYPE_INT4 = 23;
    const PG_TYPE_TEXT = 25;
    const PG_TYPE_JSON = 114;
    const PG_TYPE_FLOAT4 = 700;
    const PG_TYPE_FLOAT8 = 701;
    const PG_TYPE_VARCHAR = 1043;
}

impl sqlx::Decode<'_, sqlx::Postgres> for sql::Value {
    fn decode(
        value: <sqlx::Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        use sqlx::ValueRef;

        let type_info = value.type_info();
        match &*type_info {
            t if t.type_eq(&PG_TYPE_BOOL) => {
                <bool as PgDecode>::decode(value).map(sql::Value::Boolean)
            }
            t if t.type_eq(&PG_TYPE_FLOAT4) => {
                <f32 as PgDecode>::decode(value).map(sql::Value::Float4)
            }
            t if t.type_eq(&PG_TYPE_FLOAT8) => {
                <f64 as PgDecode>::decode(value).map(sql::Value::Float8)
            }
            t if t.type_eq(&PG_TYPE_CHAR) => <i8 as PgDecode>::decode(value).map(sql::Value::Int1),
            t if t.type_eq(&PG_TYPE_INT2) => <i16 as PgDecode>::decode(value).map(sql::Value::Int2),
            t if t.type_eq(&PG_TYPE_INT4) => <i32 as PgDecode>::decode(value).map(sql::Value::Int4),
            t if t.type_eq(&PG_TYPE_INT8) => <i64 as PgDecode>::decode(value).map(sql::Value::Int8),
            t if t.type_eq(&PG_TYPE_TEXT) => {
                <String as PgDecode>::decode(value).map(sql::Value::Text)
            }
            t if t.type_eq(&PG_TYPE_BYTEA) => {
                <Vec<u8> as PgDecode>::decode(value).map(sql::Value::Bytea)
            }

            _ => Err(Box::new(UnsupportedType::new(type_info.into_owned()))),
        }
    }
}

impl TryFrom<PgTypeInfo> for sql::PrimitiveType {
    type Error = UnsupportedType;

    fn try_from(t: PgTypeInfo) -> Result<Self, Self::Error> {
        Ok(match () {
            _ if t.type_eq(&PG_TYPE_BOOL) => Self::Boolean,
            _ if t.type_eq(&PG_TYPE_FLOAT4) => Self::Float4,
            _ if t.type_eq(&PG_TYPE_FLOAT8) => Self::Float8,
            _ if t.type_eq(&PG_TYPE_CHAR) => Self::Int1,
            _ if t.type_eq(&PG_TYPE_INT2) => Self::Int2,
            _ if t.type_eq(&PG_TYPE_INT4) => Self::Int4,
            _ if t.type_eq(&PG_TYPE_INT8) => Self::Int8,
            _ if t.type_eq(&PG_TYPE_TEXT) => Self::Text,
            _ if t.type_eq(&PG_TYPE_BYTEA) => Self::Bytea,
            _ => return Err(UnsupportedType::new(t.clone())),
        })
    }
}

impl From<sql::PrimitiveType> for PgTypeInfo {
    fn from(value: sql::PrimitiveType) -> Self {
        match value {
            sql::PrimitiveType::Boolean => PG_TYPE_BOOL,
            sql::PrimitiveType::Float4 => PG_TYPE_FLOAT4,
            sql::PrimitiveType::Float8 => PG_TYPE_FLOAT8,
            sql::PrimitiveType::Int1 => PG_TYPE_CHAR,
            sql::PrimitiveType::Int2 => PG_TYPE_INT2,
            sql::PrimitiveType::Int4 => PG_TYPE_INT4,
            sql::PrimitiveType::Int8 => PG_TYPE_INT8,
            sql::PrimitiveType::Text => PG_TYPE_TEXT,
            sql::PrimitiveType::Bytea => PG_TYPE_BYTEA,
        }
    }
}

impl From<&sql::PrimitiveType> for PgTypeInfo {
    fn from(value: &sql::PrimitiveType) -> Self {
        Self::from(*value)
    }
}

trait PgDecode<'a>: sqlx::Decode<'a, sqlx::Postgres> {
    fn decode(
        value: <sqlx::Postgres as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, sqlx::error::BoxDynError>;
}
impl<'a, T> PgDecode<'a> for T
where
    T: sqlx::Decode<'a, sqlx::Postgres>,
{
    fn decode(
        value: <sqlx::Postgres as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        <Self as sqlx::Decode<sqlx::Postgres>>::decode(value)
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
