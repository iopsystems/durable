use std::borrow::Cow;

use durable_core::bindings::sql;

use crate::driver::{Durable, TypeInfo};

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct Value(pub(crate) sql::Value);

impl Value {
    pub(crate) fn type_info(&self) -> TypeInfo {
        let ty = match &self.0 {
            sql::Value::Null(ty) => *ty,
            sql::Value::Boolean(_) => sql::PrimitiveType::Boolean,
            sql::Value::Float4(_) => sql::PrimitiveType::Float4,
            sql::Value::Float8(_) => sql::PrimitiveType::Float8,
            sql::Value::Int1(_) => sql::PrimitiveType::Int1,
            sql::Value::Int2(_) => sql::PrimitiveType::Int2,
            sql::Value::Int4(_) => sql::PrimitiveType::Int4,
            sql::Value::Int8(_) => sql::PrimitiveType::Int8,
            sql::Value::Text(_) => sql::PrimitiveType::Text,
            sql::Value::Bytea(_) => sql::PrimitiveType::Bytea,
        };

        TypeInfo(ty)
    }
}

impl sqlx::Value for Value {
    type Database = Durable;

    fn as_ref(&self) -> <Self::Database as sqlx::Database>::ValueRef<'_> {
        self
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as sqlx::Database>::TypeInfo> {
        Cow::Owned(self.type_info())
    }

    fn is_null(&self) -> bool {
        matches!(self.0, sql::Value::Null(_))
    }
}

impl<'r> sqlx::ValueRef<'r> for &'r Value {
    type Database = Durable;

    fn to_owned(&self) -> <Self::Database as sqlx::Database>::Value {
        Value::clone(self)
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as sqlx::Database>::TypeInfo> {
        <Value as sqlx::Value>::type_info(self)
    }

    fn is_null(&self) -> bool {
        <Value as sqlx::Value>::is_null(self)
    }
}
