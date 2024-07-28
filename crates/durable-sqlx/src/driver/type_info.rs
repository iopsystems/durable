use std::fmt;

use durable_core::bindings::sql;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TypeInfo(pub(crate) sql::PrimitiveType);

impl sqlx::TypeInfo for TypeInfo {
    fn is_null(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        match &self.0 {
            sql::PrimitiveType::Boolean => "bool",
            sql::PrimitiveType::Bytea => "bytea",
            sql::PrimitiveType::Float4 => "float4",
            sql::PrimitiveType::Float8 => "float8",
            sql::PrimitiveType::Int1 => "char",
            sql::PrimitiveType::Int2 => "int2",
            sql::PrimitiveType::Int4 => "int4",
            sql::PrimitiveType::Int8 => "int8",
            sql::PrimitiveType::Text => "text",
        }
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(<Self as sqlx::TypeInfo>::name(self))
    }
}
