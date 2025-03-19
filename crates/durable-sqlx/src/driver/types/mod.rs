use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::Value as _;

use super::{Durable, TypeInfo};
use crate::driver::Value;

macro_rules! generic_slice_decl {
    ($type:ty => $tyinfo:ident $ctor:ident $as_method:ident) => {
        impl<'q> sqlx::Encode<'q, Durable> for [$type] {
            fn encode_by_ref(
                &self,
                buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'q>,
            ) -> Result<IsNull, BoxDynError> {
                buf.push(Value::new(sql::Value::$ctor(self)));
                Ok(IsNull::No)
            }
        }

        impl<'q> sqlx::Decode<'q, Durable> for Vec<$type> {
            fn decode(
                value: <Durable as sqlx::Database>::ValueRef<'q>,
            ) -> Result<Self, BoxDynError> {
                if let Some(value) = value.0.$as_method() {
                    return Ok(value);
                }

                Err(unexpected_nonnull_type(
                    &<Self as sqlx::Type<Durable>>::type_info(),
                    value,
                ))
            }
        }

        impl sqlx::Type<Durable> for Vec<$type> {
            fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
                <Durable as sqlx::Database>::TypeInfo::$tyinfo()
            }
        }

        forward_slice_encode_deref!($type);
        forward_slice_type!($type);
    };
}

macro_rules! forward_encode_deref {
    ($type:ty => $target:ty) => {
        impl<'q> sqlx::Encode<'q, Durable> for $type {
            fn encode_by_ref(
                &self,
                buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'q>,
            ) -> Result<IsNull, BoxDynError> {
                <$target as sqlx::Encode<'q, Durable>>::encode_by_ref(self, buf)
            }
        }
    };
}
macro_rules! forward_slice_encode_deref {
    ($elem:ty) => {
        forward_encode_deref!(&'_ [$elem] => [$elem]);
        forward_encode_deref!(Vec<$elem> => [$elem]);
        forward_encode_deref!(Box<[$elem]> => [$elem]);
        forward_encode_deref!(std::borrow::Cow<'_, [$elem]> => [$elem]);
    }
}

macro_rules! forward_type {
    ($type:ty => $target:ty) => {
        impl sqlx::Type<Durable> for $type {
            fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
                <$target as sqlx::Type<Durable>>::type_info()
            }
        }
    };
}

macro_rules! forward_slice_type {
    ($elem:ty) => {
        forward_type!([$elem] => Vec<$elem>);
        forward_type!(Box<[$elem]> => Vec<$elem>);
        forward_type!(std::borrow::Cow<'_, [$elem]> => Vec<$elem>);
    }
}

mod boolean;
mod bytea;
#[cfg(feature = "chrono")]
mod chrono;
mod float;
mod int;
#[cfg(feature = "ipnetwork")]
mod ipnetwork;
#[cfg(feature = "json")]
mod json;
mod option;
mod text;
#[cfg(feature = "uuid")]
mod uuid;

fn unexpected_nullable_type(expected: &TypeInfo, value: &Value) -> BoxDynError {
    format!("expected {expected}, got {} instead", value.type_info()).into()
}

fn unexpected_nonnull_type(expected: &TypeInfo, value: &Value) -> BoxDynError {
    if value.is_null() {
        return format!("expected non-null {expected}, got null instead").into();
    }

    unexpected_nullable_type(expected, value)
}

fn encode_by_ref<'q, T>(
    value: &T,
    buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'q>,
) -> Result<IsNull, BoxDynError>
where
    T: sqlx::Encode<'q, Durable> + ?Sized,
{
    value.encode_by_ref(buf)
}
