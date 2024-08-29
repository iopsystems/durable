use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::bindings::durable::core::sql;
use crate::{Error, Result};

macro_rules! decl_tyinfo_ctor {
    ($( $name:ident ),* $(,)?) => {$(
        pub fn $name() -> Self {
            Self::new(sql::TypeInfo::$name())
        }
    )*};
}

#[derive(Clone, Debug)]
pub struct TypeInfo {
    tyinfo: sql::TypeInfo,
    name: String,
}

impl TypeInfo {
    pub(crate) fn new(tyinfo: sql::TypeInfo) -> Self {
        Self {
            name: tyinfo.name(),
            tyinfo,
        }
    }

    pub(crate) fn into_inner(self) -> sql::TypeInfo {
        self.tyinfo
    }

    pub fn with_name(name: &str) -> Result<Self> {
        sql::TypeInfo::with_name(name)
            .map(Self::new)
            .map_err(|_| Error::TypeNotFound {
                type_name: name.to_owned(),
            })
    }

    decl_tyinfo_ctor!(
        boolean,
        float4,
        float8,
        int1,
        int2,
        int4,
        int8,
        text,
        bytea,
        timestamptz,
        timestamp,
        uuid,
        jsonb,
        inet,
        boolean_array,
        float4_array,
        float8_array,
        int1_array,
        int2_array,
        int4_array,
        int8_array,
        text_array,
        bytea_array,
        timestamptz_array,
        timestamp_array,
        uuid_array,
        jsonb_array,
        inet_array,
    );
}

impl sqlx::TypeInfo for TypeInfo {
    fn is_null(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.tyinfo.equal(&other.tyinfo)
    }
}

impl fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

impl Serialize for TypeInfo {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        let json = self.tyinfo.serialize().map_err(Error::custom)?;
        let json: Box<str> = json.into_boxed_str();
        let json: Box<RawValue> = unsafe { std::mem::transmute(json) };

        json.serialize(ser)
    }
}

impl<'de> Deserialize<'de> for TypeInfo {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let json: Box<RawValue> = Deserialize::deserialize(de)?;
        let tyinfo = sql::TypeInfo::deserialize(json.get()).map_err(Error::custom)?;

        Ok(Self {
            name: tyinfo.name(),
            tyinfo,
        })
    }
}

impl Clone for sql::TypeInfo {
    fn clone(&self) -> Self {
        self.clone()
    }
}
