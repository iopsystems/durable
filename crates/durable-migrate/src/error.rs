#![allow(dead_code)]

use std::borrow::Cow;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::{Migrator, Options};

used_in_docs!(Migrator, Options);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(ErrorData);

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match &self.0 {
            #[cfg(feature = "migrate")]
            ErrorData::Database(_) => ErrorKind::Database,
            ErrorData::Io(_) => ErrorKind::Io,
            ErrorData::DivergingMigrations(_) => ErrorKind::DivergingMigrations,
            ErrorData::VersionOutOfRange(_) => ErrorKind::VersionOutOfRange,
            ErrorData::MissingDownMigration { .. } => todo!(),
            ErrorData::MissingTargetMigration(_) => todo!(),
            ErrorData::WouldRevert => ErrorKind::WouldRevert,
        }
    }

    pub(crate) fn io(err: std::io::Error) -> Self {
        ErrorData::Io(err).into()
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorKind {
    /// An error occurred when communicating with the database.
    ///
    /// You can get the inner [`sqlx::Error`] struct by calling the `source`
    /// method of the [`Error`].
    Database,

    /// An IO error occurred when attempting to read a migration directory.
    ///
    /// You can get at the inner [`std::io::Error`] struct by calling the
    /// `source` method of [`Error`].
    Io,

    /// The migration history applied to the database differs from the sequence
    /// of migrations that are expected to be applied to it.
    ///
    /// This can happen when either the version of the migration or the name of
    /// the migration differs between those recorded in the database and the
    /// those in the migrator.
    DivergingMigrations,

    /// A migration version number was provided that was larger than
    /// [`i64::MAX`].
    ///
    /// Migration versions are stored as `bigint`s in postgres, so version
    /// numbers larger than `i64::MAX` are invalid.
    VersionOutOfRange,

    /// Bringing the database to the requested target version would involve
    /// reverting an applied migration but that has been disallowed by the
    /// provided [`Options`].
    WouldRevert,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum ErrorData {
    #[cfg(feature = "migrate")]
    #[error(transparent)]
    Database(sqlx::Error),
    #[error(transparent)]
    Io(std::io::Error),

    #[error(transparent)]
    DivergingMigrations(DivergingMigrationError),
    #[error("migration version number was larger than i64::MAX")]
    VersionOutOfRange(std::num::TryFromIntError),
    #[error(
        "attempted to revert migration {version} {name:?} but there is no applicable down \
         migration"
    )]
    MissingDownMigration { version: u64, name: String },
    #[error("no migration with version {0}")]
    MissingTargetMigration(u64),
    #[error(
        "migrating to the target version would require reverting a migration but that is not \
         permitted"
    )]
    WouldRevert,
}

#[cfg(feature = "migrate")]
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Error(ErrorData::Database(value))
    }
}

impl From<ErrorData> for Error {
    fn from(value: ErrorData) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct DivergingMigrationError {
    pub(crate) expected_version: u64,
    pub(crate) expected_name: Cow<'static, str>,
    pub(crate) found_version: u64,
    pub(crate) found_name: String,
}

impl fmt::Display for DivergingMigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "migrations present in database diverge from expected migrations: expected migration \
             with version {} and name {:?}, found version {} and name {:?} instead",
            self.expected_version, self.expected_name, self.found_version, self.found_name
        )
    }
}

impl std::error::Error for DivergingMigrationError {}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct MigratorFromDirError(MigratorFromDirErrorData);

#[derive(Debug, thiserror::Error)]
pub(crate) enum MigratorFromDirErrorData {
    #[error("failed to list files in `{}`: {error}", path.display())]
    DirectoryIo {
        path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error("failed to read contents of `{}`: {error}", path.display())]
    FileIo {
        path: PathBuf,
        #[source]
        error: std::io::Error,
    },

    #[error(
        "migration file name `{}` contained invalid utf-8",
        Path::new(&.0).display()
    )]
    NonUtf8Filename(OsString),

    #[error("invalid migration file name `{filename}`: {reason}")]
    InvalidMigrationFilename {
        filename: String,
        reason: &'static str,
    },

    #[error("migration version for `{0}` was out of range")]
    InvalidMigrationVersion(String),

    #[error(
        "invalid extension for migration file `{0}`, only `.up.sql` and `.down.sql` are permitted"
    )]
    InvalidMigrationExt(String),

    #[error("down migration {version} is has no matching up migration")]
    MissingUpMigration { version: u64 },

    #[error("multiple migrations with version {version}: `{}` and `{}`", entry1.display(), entry2.display())]
    DuplicateMigrationVersion {
        version: u64,
        entry1: PathBuf,
        entry2: PathBuf,
    },
}

impl From<MigratorFromDirErrorData> for MigratorFromDirError {
    fn from(error: MigratorFromDirErrorData) -> Self {
        Self(error)
    }
}
