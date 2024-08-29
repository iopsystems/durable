use std::ops::Deref;

use sqlx::postgres::PgTypeInfo;

#[derive(Clone, Debug, Serialize)]
pub struct TypeInfoResource {
    pub type_info: PgTypeInfo,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
}

impl Deref for TypeInfoResource {
    type Target = PgTypeInfo;

    fn deref(&self) -> &Self::Target {
        &self.type_info
    }
}

impl From<PgTypeInfo> for TypeInfoResource {
    fn from(value: PgTypeInfo) -> Self {
        TypeInfoResource {
            type_info: value,
            name: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TypeInfoRemote {
    Resource {
        type_info: PgTypeInfo,

        #[serde(default)]
        name: Option<String>,
    },
    TypeInfo(PgTypeInfo),
}

impl<'de> serde::Deserialize<'de> for TypeInfoResource {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match TypeInfoRemote::deserialize(de)? {
            TypeInfoRemote::TypeInfo(type_info) => type_info.into(),
            TypeInfoRemote::Resource { type_info, name } => Self { type_info, name },
        })
    }
}
