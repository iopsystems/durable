use serde::{Deserialize, Serialize};
use sqlx::error::BoxDynError;
use sqlx_core::error::DatabaseError;

/// Errors that can be returned from SQLx.
///
/// This is a cut-down version of [`sqlx::Error`] that only includes the error
/// cases that can actually occur within a durable guest.
///
/// In addition, it also implements [`Serialize`] and [`Deserialize`] so that it
/// can be passed back out of a transaction.
#[non_exhaustive]
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "type", content = "error")]
#[serde(rename_all = "kebab-case")]
pub enum Error {
    /// Error returned from the database.
    #[error("error returned from database: {0}")]
    Database(
        #[source]
        #[serde(with = "database_error")]
        Box<dyn DatabaseError>,
    ),

    /// Unexpected or invalid data encountered while communicating with the
    /// database.
    ///
    /// This should indicate there is a programming error in a SQLx driver or
    /// there is something corrupted with the connection to the database
    /// itself.
    #[error("encountered unexpected or invalid data: {0}")]
    Protocol(String),

    /// No rows returned by a query that expected to return at least one row.
    #[error("no rows returned by a query that expected to return at least one row")]
    RowNotFound,

    /// Type in query doesn't exist. Likely due to typo or missing user type.
    #[error("type named {type_name} not found")]
    TypeNotFound { type_name: String },

    /// Column index was out of bounds.
    #[error("column index out of bounds: the len is {len}, but the index is {index}")]
    ColumnIndexOutOfBounds { index: usize, len: usize },

    /// No column found for the given name.
    #[error("no column found for name: {0}")]
    ColumnNotFound(String),

    /// Error occurred while decoding a value from a specific column.
    #[error("error occurred while decoding column {index}: {source}")]
    ColumnDecode {
        index: String,

        #[source]
        #[serde(with = "dyn_error")]
        source: BoxDynError,
    },

    /// Error occured while encoding a value.
    #[error("error occured while encoding a value: {0}")]
    Encode(
        #[source]
        #[serde(with = "dyn_error")]
        BoxDynError,
    ),

    /// Error occurred while decoding a value.
    #[error("error occurred while decoding: {0}")]
    Decode(
        #[source]
        #[serde(with = "dyn_error")]
        BoxDynError,
    ),

    /// A different error occurred that we were not able to handle.
    #[error(transparent)]
    #[serde(rename = "error")]
    Other(#[serde(with = "dyn_error")] BoxDynError),
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        match error {
            sqlx::Error::Database(error) => Self::Database(error),
            sqlx::Error::Protocol(error) => Self::Protocol(error),
            sqlx::Error::RowNotFound => Self::RowNotFound,
            sqlx::Error::TypeNotFound { type_name } => Self::TypeNotFound { type_name },
            sqlx::Error::ColumnIndexOutOfBounds { index, len } => {
                Self::ColumnIndexOutOfBounds { index, len }
            }
            sqlx::Error::ColumnNotFound(column) => Self::ColumnNotFound(column),
            sqlx::Error::ColumnDecode { index, source } => Self::ColumnDecode { index, source },
            sqlx::Error::Encode(error) => Self::Encode(error),
            sqlx::Error::Decode(error) => Self::Decode(error),
            error => Self::Other(error.into()),
        }
    }
}

impl<'de> Deserialize<'de> for Error {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let remote = ErrorRemote::deserialize(de)?;
        Ok(remote.into())
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "error")]
#[serde(rename_all = "kebab-case")]
enum ErrorRemote {
    Database(DynDatabaseErrorWrapper),

    Protocol(String),
    RowNotFound,
    TypeNotFound {
        type_name: String,
    },
    ColumnIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    ColumnNotFound(String),
    ColumnDecode {
        index: String,

        source: BoxDynErrorWrapper,
    },
    Encode(BoxDynErrorWrapper),
    Decode(BoxDynErrorWrapper),

    /// A different error occurred that we were not able to handle.
    #[serde(rename = "error")]
    Other(BoxDynErrorWrapper),
}

impl From<ErrorRemote> for Error {
    fn from(value: ErrorRemote) -> Self {
        match value {
            ErrorRemote::Database(error) => Self::Database(error.0),
            ErrorRemote::Protocol(error) => Self::Protocol(error),
            ErrorRemote::RowNotFound => Self::RowNotFound,
            ErrorRemote::TypeNotFound { type_name } => Self::TypeNotFound { type_name },
            ErrorRemote::ColumnIndexOutOfBounds { index, len } => {
                Self::ColumnIndexOutOfBounds { index, len }
            }
            ErrorRemote::ColumnNotFound(column) => Self::ColumnNotFound(column),
            ErrorRemote::ColumnDecode { index, source } => Self::ColumnDecode {
                index,
                source: source.0,
            },
            ErrorRemote::Encode(error) => Self::Encode(error.0),
            ErrorRemote::Decode(error) => Self::Decode(error.0),
            ErrorRemote::Other(error) => Self::Other(error.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct BoxDynErrorWrapper(#[serde(with = "dyn_error")] BoxDynError);

#[derive(Serialize, Deserialize)]
struct DynDatabaseErrorWrapper(#[serde(with = "database_error")] Box<dyn DatabaseError>);

mod database_error {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sqlx::error::{DatabaseError as SqlxDatabaseError, ErrorKind};

    use crate::bindings::durable::core::sql;
    use crate::driver::DatabaseError;

    #[allow(clippy::borrowed_box)]
    pub(crate) fn serialize<S>(
        error: &Box<dyn SqlxDatabaseError>,
        ser: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let err;
        let err = if let Some(err) = error.try_downcast_ref::<DatabaseError>() {
            &err.0
        } else {
            err = sql::DatabaseError {
                message: error.message().to_owned(),
                kind: match error.kind() {
                    ErrorKind::UniqueViolation => sql::DatabaseErrorKind::UniqueViolation,
                    ErrorKind::ForeignKeyViolation => sql::DatabaseErrorKind::ForeignKeyViolation,
                    ErrorKind::NotNullViolation => sql::DatabaseErrorKind::NotNullViolation,
                    ErrorKind::CheckViolation => sql::DatabaseErrorKind::CheckViolation,
                    _ => sql::DatabaseErrorKind::Other,
                },
                code: error.code().map(|e| e.into_owned()),
                constraint: error.constraint().map(|e| e.into()),
                table: error.table().map(|e| e.into()),
            };

            &err
        };

        err.serialize(ser)
    }

    pub(crate) fn deserialize<'de, D>(de: D) -> Result<Box<dyn SqlxDatabaseError>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Box::new(DatabaseError(sql::DatabaseError::deserialize(
            de,
        )?)))
    }
}

mod dyn_error {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use sqlx::error::BoxDynError;

    pub(crate) fn serialize<S>(error: &BoxDynError, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        error.to_string().serialize(ser)
    }

    pub(crate) fn deserialize<'de, D>(de: D) -> Result<BoxDynError, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(de)?.into())
    }
}
