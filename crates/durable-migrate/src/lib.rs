//! Migration framework for use in durable.
//!
//! This crate is designed to be a minimal migration framework for use in cases
//! where the person writing the migration is not necessarily the person running
//! the database.
//!
//! It supports the following features:
//! - The table used to store migrations is configurable. We don't control the
//!   whole database so we can't assume that any fixed migration table name
//!   (e.g. `_sqlx_migrations`) is not already being used.
//! - Migrations can be run individually in transactions, or all as one big
//!   transaction.
//! - Migrations can be reverted, but this will not automatically be done unless
//!   specifically requested.
//! - Migrations can be statically embedded via a build script so that it is not
//!   necessary for users of your library or application to keep around a bunch
//!   of migration files they may not even know about.
//!
//! # Writing migrations
//! There are two ways to create migrations:
//! 1. You can construct them directly in your codebase by creating a vec of
//!    [`Migration`] objects and create a [`Migrator`] from that.
//! 2. You can construct a [`Migrator`] from a directory using
//!    [`Migrator::from_dir`] and embed that in your application via
//!    [`Migrator::embed`], or, just directly use it.
//!
//! Embedding your migrations via a build script (approach 2) is the recommended
//! one.
//!
//! A migration directory is a directory containing migration scripts with
//! filenames of the form `<version>_<name>.[up|down].sql`. Migration version
//! numbers do not have to be contiguous (so you could have, for example, [1, 2,
//! 5] or use dates as the version numbers). However, if there is a down
//! migration then it must have a corresponding up migration.
//!
//! An example migration directory might contain
//! ```text
//! 01_do_setup.up.sql
//! 02_add_new_table_column.up.sql
//! 02_add_new_table_column.down.sql
//! ```
//!
//! This would result in two migrations:
//! - a non-revertible migration with version 1 and name "do setup", and,
//! - a revertible migration with version 2 and name "add new table column".
//!
//! # Applying migrations to a database
//! Once you have created a [`Migrator`] you can use it to migrate a database by
//! calling [`Migrator::run`]. You can control what this will do by configuring
//! the [`Options`] that you pass in. At a minimum, you will want to configure
//! [`Options::target`] and [`Options::migration_table`].
//!
//! ## Migrating to the latest version
//! ```
//! # use sqlx::Connection;
//! use durable_migrate::{Migrator, Options, Target};
//!
//! # async fn wrap() -> Result<(), Box<dyn std::error::Error>> {
//! let mut conn = sqlx::PgConnection::connect("postgres://your-database.example.com").await?;
//! let migrator = Migrator::from_dir("migrations")?;
//! let options = Options {
//!     target: Target::Latest,
//!     ..Options::default()
//! };
//!
//! migrator.run(&mut conn, &options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Migrating to a specific version
//! Note that we set `allow_revert` to true here so that down migrations can
//! be applied if necessary.
//! ```
//! # use sqlx::Connection;
//! use durable_migrate::{Migrator, Options, Target};
//!
//! # async fn wrap() -> Result<(), Box<dyn std::error::Error>> {
//! let mut conn = sqlx::PgConnection::connect("postgres://your-database.example.com").await?;
//! let migrator = Migrator::from_dir("migrations")?;
//! let options = Options {
//!     target: Target::Version(5),
//!     allow_revert: true,
//!     ..Options::default()
//! };
//!
//! migrator.run(&mut conn, &options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Testing Migrations
//! By setting `dry_run` to true we can run all the migrations in one big
//! transaction to verify that they work, then roll the transaction back at the
//! end so that no changes are made.
//!
//! Note that this will still take locks in the database, so other connections
//! may hang while a dry run is running.
//! ```
//! # use sqlx::Connection;
//! use durable_migrate::{Migrator, Options, Target};
//!
//! # async fn wrap() -> Result<(), Box<dyn std::error::Error>> {
//! let mut conn = sqlx::PgConnection::connect("postgres://your-database.example.com").await?;
//! let migrator = Migrator::from_dir("migrations")?;
//! let options = Options {
//!     target: Target::Version(5),
//!     dry_run: true,
//!     ..Options::default()
//! };
//!
//! migrator.run(&mut conn, &options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Embedding migrations
//! This crate supports embedding migrations via a build script. To do so, you
//! will want a build script that looks roughly like
//! ```no_run
//! use std::path::PathBuf;
//!
//! use durable_migrate::{EmbedOptions, Migrator};
//!
//! fn main() {
//!     # let tempdir = tempdir::TempDir::new("durable").expect("failed to create temp dir");
//!     # let out_dir = tempdir.path();
//!     # if cfg!(any()) {
//!     let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
//!     # }
//!     let migrations = Migrator::from_dir("migrations").expect("failed to load migrations");
//!     let embed = migrations.embed(&EmbedOptions::default());
//!
//!     std::fs::write(out_dir.join("migrations.rs"), &embed);
//! }
//! ```
//!
//! Then, you can include the migrations by doing
//! ```no_compile
//! include!(concat!(env!("OUT_DIR"), "/migrations.rs"));
//!
//! let migrator = MIGRATOR;
//! ```
//!
//! You can configure some options on how the migrations are embedded via
//! [`EmbedOptions`]. You will also want to disable default features for
//! `durable-migrate` when using it as a build dependency so that you don't pull
//! sqlx in as a build dependency.

#![allow(clippy::needless_doctest_main)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Helper macro used to silence `unused_import` warnings when an item is
/// only imported in order to refer to it within a doc comment.
macro_rules! used_in_docs {
    ($( $item:ident ),*) => {
        const _: () = {
            #[allow(unused_imports)]
            mod dummy {
                $( use super::$item; )*
            }
        };
    };
}

#[cfg(feature = "migrate")]
mod apply;
mod error;

pub use self::error::{DivergingMigrationError, Error, ErrorKind, MigratorFromDirError};

/// The migration target version that we want to bring the database to.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Target {
    /// Migrate to the latest available version.
    ///
    /// This will never perform any reverts on the database.
    ///
    /// Note that this is different from
    /// `Target::Version(migrator.latest().unwrap())` since it will not revert
    /// migrations applied to the database even if they don't exist within the
    /// current migrator.
    Latest,

    /// Migrate to a specific version.
    Version(u64),
}

/// Controls how migrations are run in transactions.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TransactionMode {
    /// Run all applied migrations in a single transaction.
    Single,

    /// Run each migration in its own transaction.
    Individual,
}

/// Describes a table in SQL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    /// The schema the table is in.
    ///
    /// If not provided then this will place the table into the default search
    /// path. This usually means putting it in the `public` schema.
    pub schema: Option<Cow<'static, str>>,

    /// The name of the table.
    pub name: Cow<'static, str>,
}

impl Table {
    /// A table with both schema and name.
    pub fn new(schema: impl Into<Cow<'static, str>>, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            schema: Some(schema.into()),
            name: name.into(),
        }
    }

    /// A table with no schema specified.
    pub fn plain(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            schema: None,
            name: name.into(),
        }
    }

    #[cfg_attr(not(feature = "migrate"), allow(dead_code))]
    fn as_sql(&self) -> String {
        match self.schema.as_deref() {
            Some(schema) => format!("{schema:?}.{:?}", self.name),
            None => format!("{:?}", self.name),
        }
    }
}

/// Options controlling how migrations are run and what they are allowed to do.
#[derive(Debug, Clone)]
pub struct Options {
    /// Whether revert migrations are permitted to run.
    ///
    /// This is `false` by default.
    pub allow_revert: bool,

    /// Make all the changes to the database inside one big transaction but
    /// don't actually commit that transaction at the end.
    ///
    /// Setting this to true overrides `transaction_mode` to be
    /// [`TransactionMode::Single`].
    pub dry_run: bool,

    /// Prefer using the local revert migration over those saved in the
    /// database, where available.
    ///
    /// This is relevant in cases where the revert script in the database is
    /// incorrect, but a fixed on is available to the current version of the
    /// application.
    pub prefer_local_revert: bool,

    /// The target database version to migrate to.
    pub target: Target,

    /// How migrations are run within transactions.
    pub transaction_mode: TransactionMode,

    /// The table to store migration data in.
    ///
    /// If this is ever changed, then the migration framework will forget all
    /// database migrations that have been previously applied and will attempt
    /// to migrate again from scratch. This will break your database.
    ///
    /// By default this is `migrations`.
    pub migration_table: Table,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            allow_revert: false,
            dry_run: false,
            prefer_local_revert: true,
            target: Target::Latest,
            migration_table: Table::plain("migrations"),
            transaction_mode: TransactionMode::Single,
        }
    }
}

/// A single migration.
#[derive(Clone, Debug)]
pub struct Migration {
    pub version: u64,
    pub name: Cow<'static, str>,
    pub sql: Cow<'static, str>,
    pub revert: Option<Cow<'static, str>>,
}

struct MigrationSource {
    up: PathBuf,
    down: Option<PathBuf>,
}

struct MigratorSources {
    migrations: Vec<MigrationSource>,
    directory: PathBuf,
}

/// A collection of migrations that can be applied to the database.
pub struct Migrator {
    migrations: Cow<'static, [Migration]>,
    sources: Option<Box<MigratorSources>>,
}

impl Migrator {
    /// Create a migrator from a list of migrations.
    pub fn new(mut migrations: Vec<Migration>) -> Self {
        migrations.sort_by_key(|migration| migration.version);

        Self {
            migrations: migrations.into(),
            sources: None,
        }
    }

    /// Create a migrator from a static array of migrations.
    pub const fn from_static(migrations: &'static [Migration]) -> Self {
        // We use a while loop here because iterators are not yet const.
        let mut i = 0;
        let mut prev = None;
        while i < migrations.len() {
            let migration = &migrations[i];

            if let Some(prev) = prev {
                if migration.version <= prev {
                    panic!("migrations are not sorted");
                }
            }

            prev = Some(migration.version);
            i += 1;
        }

        Self {
            migrations: Cow::Borrowed(migrations),
            sources: None,
        }
    }

    /// Load migrations from a directory.
    ///
    /// This scans the directory at `path` for files of the form
    /// `<version>_<name>.[up|down].sql` and uses that to create a [`Migrator`].
    /// Files that do not end with a `.sql` extension are silently ignored.
    ///
    /// # Errors
    /// This method will emit an error if:
    /// - There is an IO error while reading the directory.
    /// - There is an IO error while reading an individual migration file.
    /// - There are multiple migrations with the same version number but
    ///   different names. This includes an up/down migration pair with
    ///   mismatching names.
    /// - There is a down migration without a corresponding up migration.
    /// - There is a file ending in `.sql` that doesn't match the pattern above.
    pub fn from_dir<P: AsRef<Path>>(path: P) -> Result<Self, MigratorFromDirError> {
        Self::_from_dir(path.as_ref())
    }

    fn _from_dir(path: &Path) -> Result<Self, MigratorFromDirError> {
        use crate::error::MigratorFromDirErrorData as Error;

        struct MigrationEntry {
            version: u64,
            name: String,
            path: PathBuf,
        }

        let mut up: BTreeMap<u64, MigrationEntry> = BTreeMap::new();
        let mut down: BTreeMap<u64, MigrationEntry> = BTreeMap::new();

        for entry in std::fs::read_dir(path).map_err(|error| Error::DirectoryIo {
            path: path.to_path_buf(),
            error,
        })? {
            let entry = entry.map_err(|error| Error::DirectoryIo {
                path: path.to_path_buf(),
                error,
            })?;
            let file_name = entry.file_name();
            let Some(file_name) = file_name.to_str() else {
                return Err(Error::NonUtf8Filename(file_name).into());
            };

            let Some((stem, ext)) = file_name.split_once(".") else {
                continue;
            };

            let is_up = match ext {
                "up.sql" => true,
                "down.sql" => false,
                _ if ext == "sql" || ext.ends_with(".sql") => {
                    return Err(Error::InvalidMigrationExt(file_name.into()).into())
                }
                _ => continue,
            };

            let Some((version, name)) = stem.split_once("_") else {
                return Err(Error::InvalidMigrationFilename {
                    filename: file_name.into(),
                    reason: "missing `_` separator between the version number and migration name",
                }
                .into());
            };

            let version: u64 = version
                .parse()
                .map_err(|_| Error::InvalidMigrationFilename {
                    filename: file_name.into(),
                    reason: "version number could not be parsed",
                })?;

            if version > i64::MAX as u64 {
                return Err(Error::InvalidMigrationVersion(file_name.into()).into());
            }

            let name = name.replace('_', " ");
            let entry = MigrationEntry {
                version,
                name,
                path: entry.path(),
            };

            let map = if is_up { &mut up } else { &mut down };
            if let Some(prev) = map.get(&entry.version) {
                return Err(Error::DuplicateMigrationVersion {
                    version: entry.version,
                    entry1: prev.path.clone(),
                    entry2: entry.path.clone(),
                }
                .into());
            }

            map.insert(entry.version, entry);
        }

        for (&version, down) in down.iter() {
            let Some(up) = up.get(&version) else {
                return Err(Error::MissingUpMigration { version }.into());
            };

            if up.name != down.name {
                return Err(Error::DuplicateMigrationVersion {
                    version,
                    entry1: up.path.clone(),
                    entry2: down.path.clone(),
                }
                .into());
            }
        }

        let mut migrations = Vec::new();
        let mut sources = Vec::new();
        for (version, up) in up.into_iter() {
            let sql = std::fs::read_to_string(&up.path).map_err(|e| Error::FileIo {
                path: up.path.clone(),
                error: e,
            })?;

            let (revert, down) = match down.get(&version) {
                Some(down) => {
                    let sql = std::fs::read_to_string(&down.path).map_err(|e| Error::FileIo {
                        path: down.path.clone(),
                        error: e,
                    })?;

                    (Some(sql), Some(down.path.clone()))
                }
                None => (None, None),
            };

            migrations.push(Migration {
                version,
                name: Cow::Owned(up.name.clone()),
                sql: Cow::Owned(sql),
                revert: revert.map(Cow::Owned),
            });

            sources.push(MigrationSource { up: up.path, down });
        }

        Ok(Self {
            migrations: migrations.into(),
            sources: Some(Box::new(MigratorSources {
                migrations: sources,
                directory: path.to_owned(),
            })),
        })
    }

    /// Returns the version number of the latest migration contained in this
    /// migrator.
    pub fn latest(&self) -> Option<u64> {
        self.migrations.last().map(|migration| migration.version)
    }

    /// Get all the migrations contained within this migrator.
    pub fn migrations(&self) -> &[Migration] {
        &self.migrations
    }
}

/// Options controlling the output of [`Migrator::embed`].
#[derive(Clone, Debug)]
pub struct EmbedOptions {
    /// The name to use for the generated rust constant.
    ///
    /// By default, this is `MIGRATIONS`.
    pub name: Cow<'static, str>,

    /// Whether the contents of the migration SQL files should be embedded
    /// directly, as strings, or via `include_str!`.
    ///
    /// Generally, `include_str!` is nicer when used in a build script, but if
    /// you're not sure the migration directory will be around when building the
    /// library then you'll want to set this to false.
    ///
    /// By default, this is true.
    pub use_includes: bool,

    /// Emit directives that tell cargo to rerun the build script if the
    /// migrations directory changes.
    ///
    /// By default, this is true.
    pub print_cargo_directives: bool,

    /// Path that the `durable_migrate` crate can be accessed from.
    ///
    /// Defaults to `::durable_migrate`
    pub crate_path: Cow<'static, str>,
}

impl Default for EmbedOptions {
    fn default() -> Self {
        Self {
            name: "MIGRATIONS".into(),
            use_includes: true,
            print_cargo_directives: true,
            crate_path: "::durable_migrate".into(),
        }
    }
}

impl Migrator {
    /// Generate code for a rust constant containing all migrations in this
    /// migrator.
    ///
    /// This is designed to be used from build scripts, so that the migrations
    /// are embedded at build time.
    pub fn embed(&self, options: &EmbedOptions) -> String {
        use std::fmt::Write;

        let mut content = String::new();
        let sources = self.sources.as_deref();

        let include_path = |path: &Path| {
            format!(
                r#"include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", {:?}))"#,
                path.display().to_string()
            )
        };

        write!(
            content,
            "\
pub const {name}: {path}::Migrator = {path}::Migrator::from_static({{
    use ::std::borrow::Cow;

    &[
",
            name = options.name,
            path = options.crate_path
        )
        .unwrap();

        for (idx, migration) in self.migrations.iter().enumerate() {
            let source = sources.and_then(|sources| sources.migrations.get(idx));

            let up = source
                .filter(|_| options.use_includes)
                .map(|source| include_path(&source.up))
                .unwrap_or_else(|| format!("{:?}", migration.sql));
            let down = match migration.revert.as_deref() {
                Some(revert) => {
                    let down = source
                        .filter(|_| options.use_includes)
                        .and_then(|source| source.down.as_deref())
                        .map(include_path)
                        .unwrap_or_else(|| format!("{:?}", revert));

                    format!("Some(Cow::Borrowed({down}))")
                }
                None => "None".into(),
            };

            write!(
                content,
                "       {path}::Migration {{
            version: {version},
            name: Cow::Borrowed({name:?}),
            sql: Cow::Borrowed({up}),
            revert: {down},
        }},
",
                path = options.crate_path,
                version = migration.version,
                name = migration.name,
            )
            .unwrap();

            if !options.use_includes && options.print_cargo_directives {
                if let Some(source) = source {
                    println!("cargo:rerun-if-changed={}", source.up.display());

                    if let Some(down) = source.down.as_deref() {
                        println!("cargo:rerun-if-changed={}", down.display());
                    }
                }
            }
        }

        write!(
            content,
            "    ]
}});
"
        )
        .unwrap();

        if options.print_cargo_directives {
            if let Some(sources) = &self.sources {
                println!("cargo:rerun-if-changed={}", sources.directory.display());
            }
        }

        content
    }
}
