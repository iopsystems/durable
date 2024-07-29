use std::fmt;

use crate::bindings as sql;

#[derive(Clone, Debug, Default)]
pub(crate) struct UnsupportedError(());

impl UnsupportedError {
    pub fn new() -> Self {
        Self(())
    }
}

impl fmt::Display for UnsupportedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("operation not supported")
    }
}

impl std::error::Error for UnsupportedError {}

#[derive(Debug)]
struct DatabaseError(sql::DatabaseError);

impl sqlx::error::DatabaseError for DatabaseError {
    fn message(&self) -> &str {
        &self.0.message
    }

    fn kind(&self) -> sqlx::error::ErrorKind {
        use sqlx::error::ErrorKind;

        match self.0.kind {
            sql::DatabaseErrorKind::UniqueViolation => ErrorKind::UniqueViolation,
            sql::DatabaseErrorKind::ForeignKeyViolation => ErrorKind::ForeignKeyViolation,
            sql::DatabaseErrorKind::NotNullViolation => ErrorKind::NotNullViolation,
            sql::DatabaseErrorKind::CheckViolation => ErrorKind::CheckViolation,
            _ => ErrorKind::Other,
        }
    }

    fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
        self
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.message.fmt(f)
    }
}

impl std::error::Error for DatabaseError {}

struct StringError(String);

impl fmt::Debug for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for StringError {}

pub(crate) fn convert_query_error(err: sql::Error) -> sqlx::Error {
    match err {
        sql::Error::ColumnDecode(e) => sqlx::Error::ColumnDecode {
            index: e.index,
            source: Box::new(StringError(e.source)),
        },
        sql::Error::Encode(e) => sqlx::Error::Encode(Box::new(StringError(e))),
        sql::Error::Decode(e) => sqlx::Error::Decode(Box::new(StringError(e))),
        sql::Error::Database(e) => sqlx::Error::Database(Box::new(DatabaseError(e))),
        sql::Error::TypeNotFound(type_name) => sqlx::Error::TypeNotFound { type_name },
        sql::Error::Other(e) => sqlx::Error::Protocol(e),
    }
}
