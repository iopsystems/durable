use std::str::FromStr;

use anyhow::Context;
use durable_migrate::{Migrator, Options, Table, Target, TransactionMode};
use sqlx::Connection;

/// Apply migrations to the database.
#[derive(Debug, clap::Args)]
pub struct Migrate {
    #[command(subcommand)]
    command: Command,

    /// Whether reverting migrations is permitted.
    ///
    /// If not set, then attempting to perform a migration that would require
    /// reverting will instead cause an error. This is implied by the revert
    /// command.
    #[arg(long)]
    allow_revert: bool,

    /// Run all the migrations in a transaction but don't actually commit them
    /// to the database.
    ///
    /// This allows you to test that the migration works as expected without
    /// having to worry about wrecking the database.
    #[arg(long)]
    dry_run: bool,

    /// The target version to migrate to.
    #[arg(long)]
    target: Option<u64>,

    /// URL of the database to connect to.
    #[arg(long)]
    database_url: Option<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Migrate to a target version.
    Apply,

    /// Revert to a target version.
    ///
    /// This is effectively just an alias to apply that implies --allow-revert.
    /// By default, it will revert the latest migration in the database.
    Revert,

    /// Recreate the database then migrate to a target version.
    Reset,
}

impl Migrate {
    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) -> anyhow::Result<()> {
        self._run().await
    }

    async fn _run(self) -> anyhow::Result<()> {
        let root = crate::workspace_root()?;
        let migrator = Migrator::from_dir(root.join("crates/durable-runtime/migrations"))?;

        let database_url = match self.database_url {
            Some(url) => Some(url),
            None => dotenvy::var("DATABASE_URL").ok(),
        };

        let options = match database_url.as_deref() {
            Some(url) => sqlx::postgres::PgConnectOptions::from_str(url)
                .context("failed to parse the provided database url")?,
            None => sqlx::postgres::PgConnectOptions::new(),
        };
        let mut conn = sqlx::PgConnection::connect_with(&options)
            .await
            .context("failed to connect to the database")?;

        let table = Table::new("durable", "migrations");
        let target = match self.target {
            Some(target) => Target::Version(target),
            None => match self.command {
                Command::Apply => Target::Latest,
                Command::Reset => Target::Latest,
                Command::Revert => {
                    let latest = match migrator.read_database_version(&mut conn, &table).await? {
                        Some(version) => version,
                        None => anyhow::bail!("database has no migrations applied"),
                    };

                    let prev = migrator
                        .migrations()
                        .iter()
                        .rev()
                        .skip_while(|migration| migration.version >= latest)
                        .next()
                        .map(|migration| migration.version)
                        .unwrap_or(0);

                    Target::Version(prev)
                }
            },
        };

        let options = Options {
            target,
            transaction_mode: TransactionMode::Single,
            allow_revert: self.allow_revert || matches!(self.command, Command::Revert),
            dry_run: self.dry_run,
            migration_table: table,
            prefer_local_revert: true,
            ..Default::default()
        };

        if matches!(self.command, Command::Reset) {
            sqlx::query("DROP SCHEMA IF EXISTS durable CASCADE")
                .execute(&mut conn)
                .await
                .context("failed to drop schema durable")?;
        }

        migrator.run(&mut conn, &options).await?;

        Ok(())
    }
}
