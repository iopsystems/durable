use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::bindings as sql;
use crate::driver::{Durable, TypeInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct Value(pub(crate) sql::Value);

impl Value {
    pub(crate) fn new(value: sql::Value) -> Self {
        Self(value)
    }

    pub fn type_info(&self) -> TypeInfo {
        TypeInfo::new(self.0.type_info())
    }
}

impl sqlx::Value for Value {
    type Database = Durable;

    fn as_ref(&self) -> <Self::Database as sqlx::Database>::ValueRef<'_> {
        self
    }

    fn type_info(&self) -> Cow<'_, <Self::Database as sqlx::Database>::TypeInfo> {
        Cow::Owned(TypeInfo::new(self.0.type_info()))
    }

    fn is_null(&self) -> bool {
        self.0.is_null()
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

impl Serialize for sql::Value {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        let json = self.serialize().map_err(Error::custom)?;
        let json: Box<str> = json.into_boxed_str();
        let json: Box<RawValue> = unsafe { std::mem::transmute(json) };

        json.serialize(ser)
    }
}

impl<'de> Deserialize<'de> for sql::Value {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let json: Box<RawValue> = Deserialize::deserialize(de)?;
        let value = sql::Value::deserialize(json.get()).map_err(Error::custom)?;

        Ok(value)
    }
}

impl Clone for sql::Value {
    fn clone(&self) -> Self {
        self.clone()
    }
}
