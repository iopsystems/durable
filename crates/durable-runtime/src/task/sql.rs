use async_trait::async_trait;
use sqlx::postgres::PgTypeInfo;

use super::WorkflowState;
use crate::bindings::durable::sql;

#[async_trait]
impl sql::Host for WorkflowState {
    async fn query(
        &mut self,
        sql: String,
        params: Vec<sql::Value>,
        options: sql::Options,
    ) -> anyhow::Result<Result<Vec<sql::Row>, sql::Error>> {
        todo!()
    }
}

macro_rules! primitive_for_each {
    {
        match $self:expr => {
            $var:ident => $result:expr,
            null => $null:expr $(,)?
        }
    } => {
        match $self {
            Self::Null => $null,
            Self::Boolean($var) => $result,
            Self::Single($var) => $result,
            Self::Double($var) => $result,
            Self::Int8($var) => $result,
            Self::Int16($var) => $result,
            Self::Int32($var) => $result,
            Self::Int64($var) => $result,
            Self::Text($var) => $result,
            Self::Bytea($var) => $result,
        }
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Postgres> for sql::Primitive {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&v, buf),
                null => Ok(sqlx::encode::IsNull::Yes)
            }
        }
    }

    fn encode(
        self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::encode(v, buf),
                null => Ok(sqlx::encode::IsNull::Yes)
            }
        }
    }

    fn produces(&self) -> Option<<sqlx::Postgres as sqlx::Database>::TypeInfo> {
        primitive_for_each! {
            match &self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::produces(v),
                null => None
            }
        }
    }

    fn size_hint(&self) -> usize {
        primitive_for_each! {
            match self => {
                v => <_ as sqlx::Encode<sqlx::Postgres>>::size_hint(v),
                null => 0
            }
        }
    }
}

impl<'a> sqlx::Encode<'a, sqlx::Postgres> for sql::Value {
    fn encode_by_ref(
            &self,
            buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'a>,
        ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            Self::Primitive(v) => v.encode_by_ref(buf),
            Self::Array(v) => v.encode_by_ref(buf),
        }
    }
}
