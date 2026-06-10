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
                buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
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
                buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
            ) -> Result<IsNull, BoxDynError> {
                <$target as sqlx::Encode<'q, Durable>>::encode_by_ref(self, buf)
            }
        }
    };
}
macro_rules! forward_slice_encode_deref {
    ($elem:ty) => {
        // sqlx 0.9's blanket `impl Encode for &T` requires `T: Sized`, so it does
        // *not* cover `&[$elem]` (the pointee `[$elem]` is unsized). Forward it here
        // explicitly so slice references stay bindable. `Cow<'_, [$elem]>` is covered
        // by a blanket `Encode` impl in sqlx 0.9.
        forward_encode_deref!(&'_ [$elem] => [$elem]);
        forward_encode_deref!(Vec<$elem> => [$elem]);
        forward_encode_deref!(Box<[$elem]> => [$elem]);
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
        // `Box<T>` and `Cow<'_, T>` are covered by blanket `Type` impls in sqlx 0.9,
        // which forward to `<[$elem] as Type>::type_info()` via this base impl.
        forward_type!([$elem] => Vec<$elem>);
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
    buf: &mut <Durable as sqlx::Database>::ArgumentBuffer,
) -> Result<IsNull, BoxDynError>
where
    T: sqlx::Encode<'q, Durable> + ?Sized,
{
    value.encode_by_ref(buf)
}

#[cfg(test)]
mod tests {
    use super::Durable;

    /// Regression test for #114: binding a `&[T]` slice reference as an array
    /// parameter must compile for every element type declared via the slice
    /// macros, matching the hand-written `uuid` impls. sqlx 0.9's blanket
    /// `impl Encode for &T` requires `T: Sized`, so it does not cover `&[T]`
    /// (the pointee `[T]` is unsized) — the forwarding impls are what make
    /// these bindable.
    fn assert_encode<'q, T: sqlx::Encode<'q, Durable>>() {}

    #[test]
    fn slice_refs_implement_encode() {
        assert_encode::<&[i8]>();
        assert_encode::<&[i16]>();
        assert_encode::<&[i32]>();
        assert_encode::<&[i64]>();
        assert_encode::<&[f32]>();
        assert_encode::<&[f64]>();
        assert_encode::<&[bool]>();
        assert_encode::<&[&str]>();
        assert_encode::<&[String]>();
        #[cfg(feature = "uuid")]
        assert_encode::<&[::uuid::Uuid]>();
    }
}
