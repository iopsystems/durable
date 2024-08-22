//! Database migrations for the durable runtime.

use durable_migrate::Table;

use self::migrations::MIGRATIONS;

mod migrations {
    include!(concat!(env!("OUT_DIR"), "/migrations.rs"));
}

#[doc(inline)]
pub use durable_migrate::{
    DivergingMigrationError, Error, ErrorKind, Options, Target, TransactionMode,
};

/// A migrator that comes pre-loaded with migrations relevant to durable.
pub struct Migrator(durable_migrate::Migrator);

impl Default for Migrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Migrator {
    /// Create a migrator with migrations for durable.
    pub const fn new() -> Self {
        Self(MIGRATIONS)
    }

    /// Get a [`Target`] that points to the latest version supported by this
    /// migrator.
    ///
    /// Note that this may be older than the version of the latest migration
    /// applied to the database if the runtime has been downgraded.
    pub fn latest(&self) -> Target {
        Target::Version(self.latest_version())
    }

    /// Get the version number of the latest migration supported by this
    /// migrator.
    ///
    /// Note that this may be older than the version of the latest migration
    /// applied to the database if the runtime has been downgraded.
    pub fn latest_version(&self) -> u64 {
        self.0.latest().unwrap()
    }

    /// Migrate the database.
    ///
    /// This will mostly follow the configuration as requested in [`Options`]
    /// but will override the table used to store migration data.
    pub async fn migrate(
        &self,
        conn: &mut sqlx::PgConnection,
        options: &Options,
    ) -> Result<(), Error> {
        let mut options = options.clone();

        // Note that changing this means that all previously applied migrations in the
        // database will be forgotten.
        options.migration_table = Table::new("durable", "migrations");

        self.0.run(conn, &options).await
    }

    /// Read the latest migration version applied to the database.
    pub async fn read_database_version(
        &self,
        conn: &mut sqlx::PgConnection,
    ) -> Result<Option<u64>, Error> {
        let table = Table::new("durable", "migrations");
        self.0.read_database_version(conn, &table).await
    }
}
