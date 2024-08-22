pub mod exports {
    pub use durable_sqlx_macros::expand_query;

    use crate::driver::Durable;

    pub mod sqlx {
        pub use sqlx::*;

        pub mod postgres {
            pub use crate::driver::{Durable as Postgres, Row as PgRow};
        }
    }

    pub fn into_durable<T: IntoDurable>(value: T) -> T::Durable {
        value.into_durable()
    }

    pub trait IntoDurable {
        type Durable;

        fn into_durable(self) -> Self::Durable;
    }

    impl<'q, F, A> IntoDurable for sqlx::query::Map<'q, Durable, F, A> {
        type Durable = crate::Map<'q, F, A>;

        fn into_durable(self) -> Self::Durable {
            crate::Map(self)
        }
    }

    impl<'q, A> IntoDurable for sqlx::query::Query<'q, Durable, A> {
        type Durable = crate::Query<'q, A>;

        fn into_durable(self) -> Self::Durable {
            crate::Query(self)
        }
    }

    impl<'q, O, A> IntoDurable for sqlx::query::QueryAs<'q, Durable, O, A> {
        type Durable = crate::QueryAs<'q, O, A>;

        fn into_durable(self) -> Self::Durable {
            crate::QueryAs(self)
        }
    }

    impl<'q, O, A> IntoDurable for sqlx::query::QueryScalar<'q, Durable, O, A> {
        type Durable = crate::QueryScalar<'q, O, A>;

        fn into_durable(self) -> Self::Durable {
            crate::QueryScalar(self)
        }
    }
}

/// Statically checked SQL query with `println!()` style syntax.
///
/// This is a thin wrapper around [`sqlx::query!`]. See the docs there for usage
/// details.
#[macro_export]
macro_rules! query {
    ($query:expr $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            source = $query
            $(, args = [$($args)*])?
        ))
    }}
}

/// A variant of [`query!`] which does not check the input or output types. This
/// still does parse the query to ensure it's syntactically and semantically
/// valid for the current database.
#[macro_export]
macro_rules! query_unchecked {
    ($query:expr $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            source = $query,
            $(args = [$($args)*],)?
            checked = false
        ))
    }}
}

/// A variant of [`query!`] where the SQL query is stored in a separate file.
///
/// Useful for large queries and potentially cleaner than multiline strings.
///
/// The syntax and requirements (see [`query!`]) are the same except the string
/// is replaced by a file path.
///
/// The file must be relative to the project root (the directory containing
/// `Cargo.toml`), unlike `include_str!()` which uses compiler internals to get
/// the path of the file where it was invoked.
#[macro_export]
macro_rules! query_file {
    ($path:literal $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            source_file = $path
            $(, args = [$($args)*])?
        ))
    }}
}

/// A variant of [`query_file!`] which does not check the input or output types.
///
/// This still does parse the query to ensure it's syntactically and
/// semantically valid for the current database.
#[macro_export]
macro_rules! query_file_unchecked {
    ($path:literal $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            source_file = $path,
            $(args = [$($args)*],)?
            checked = false
        ))
    }}
}

/// A variant of [`query!`] which takes a path to an explicitly defined struct
/// as the output type.
///
/// This is a wrapper around [`sqlx::query_as!`], see the docs there for more.
#[macro_export]
macro_rules! query_as {
    ($out_struct:path, $query:expr $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            record = $out_struct,
            source = $query
            $(, args = [$($args)*])?
        ))
    }}
}

/// Combines the syntax of [`query_as!`] and [`query_file!`].
///
/// Enforces requirements of both macros; see them for details.
#[macro_export]
macro_rules! query_file_as {
    ($out_struct:path, $path:literal $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            record = $out_struct,
            source_file = $path
            $(, args = [$($args)*])?
        ))
    }}
}

/// A variant of [`query_as!`] which does not check the input or output types.
///
/// This still does parse the query to ensure it's syntactically and
/// semantically valid for the current database.
#[macro_export]
macro_rules! query_as_unchecked {
    (out_struct:path, $query:expr $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            record = $out_struct,
            source = $query,
            $( args = [$($args)*],)?
            checked = false
        ))
    }}
}

/// A variant of [`query_file_as!`] which does not check the input or output
/// types.
///
/// This still does parse the query to ensure it's syntactically and
/// semantically valid for the current database.
#[macro_export]
macro_rules! query_file_as_unchecked {
    ($out_struct:path, $path:literal $(, $($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            record = $out_struct,
            source_file = $path,
            $(args = [$($args)*],)?
            checked = false
        ))
    }}
}

/// A variant of [`query!`] which expects a single column from the query and
/// evaluates to an instance of [`QueryScalar`](crate::QueryScalar).
///
/// See [`sqlx::query_scalar!`] for more details.
#[macro_export]
macro_rules! query_scalar {
    ($query:expr $(, $($args:tt)*)?) =>  {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            scalar = _,
            source = $query
            $(, args = [$($args)*])?
        ))
    }}
}

/// A variant of [`query_scalar!`] which takes a file path like [`query_file!`].
#[macro_export]
macro_rules! query_file_scalar {
    ($path:literal $(,$($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            scalar = _,
            source_file = $path
            $(, args = [$($args)*])?
        ))
    }}
}

/// A variant of [`query_scalar!`] which does not typecheck bind parameters and
/// leaves the output type to inference.
///
/// See [`sqlx::query_scalar_unchecked!`] for more details.
#[macro_export]
macro_rules! query_scalar_unchecked {
    ($query:expr $(,$($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            scalar = _,
            source = $query,
            $(args = [$($args)*],)?
            checked = false
        ))
    }}
}

/// A variant of [`query_file_scalar!`] which does not typecheck bind parameters
/// and leaves the output type to inference.
///
/// See [`sqlx::query_scalar_unchecked!`] for more details.
#[macro_export]
macro_rules! query_file_scalar_unchecked {
    ($path:literal $(,$($args:tt)*)?) => {{
        use $crate::exports::sqlx;

        $crate::exports::into_durable($crate::exports::expand_query!(
            scalar = _,
            source_file = $path,
            $(args = [$($args)*],)?
            checked = false
        ))
    }}
}
