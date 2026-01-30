use std::time::Duration;

use anyhow::Context;
use durable_migrate::{Migrator, Options, Table, Target, TransactionMode};

#[derive(Debug, clap::Args)]
pub struct Claude {
    #[command(subcommand)]
    command: ClaudeCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum ClaudeCommand {
    /// Set up the local PostgreSQL database for development.
    ///
    /// This starts the locally-installed PostgreSQL server, configures it for
    /// trust authentication, runs migrations, and writes a .env file with the
    /// DATABASE_URL. Useful in environments where Docker is not available
    /// (e.g. CI containers with a system PostgreSQL installation).
    Setup(Setup),
}

#[derive(Debug, clap::Args)]
pub struct Setup {
    /// Disallow overwriting a pre-existing environment file.
    #[arg(long)]
    no_overwrite_env: bool,
}

impl Claude {
    pub fn run(self) -> anyhow::Result<()> {
        match self.command {
            ClaudeCommand::Setup(cmd) => cmd.run(),
        }
    }
}

impl Setup {
    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) -> anyhow::Result<()> {
        let root = crate::workspace_root()?;
        let migrator = Migrator::from_dir(root.join("crates/durable-runtime/migrations"))?;
        let sh = xshell::Shell::new()?;

        // Ensure PostgreSQL is running with the right auth config.
        Self::ensure_postgres(&sh)?;

        // Wait for PostgreSQL to accept connections.
        let pool = self.connect().await?;

        // Run migrations.
        if let Err(e) = Self::migrate(&pool, &migrator).await {
            pool.close().await;
            anyhow::bail!("migrating the database failed: {e:?}");
        }

        pool.close().await;

        // Write the .env file.
        let env = root.join(".env");
        if !env.exists() || !self.no_overwrite_env {
            std::fs::write(
                &env,
                b"DATABASE_URL=postgres://postgres@localhost:5432/postgres\n",
            )
            .context("failed to write the .env file")?;
            tracing::info!("wrote .env file");
        } else {
            tracing::warn!("refusing to overwrite existing .env file");
        }

        tracing::info!("database setup complete");
        Ok(())
    }

    fn ensure_postgres(sh: &xshell::Shell) -> anyhow::Result<()> {
        // Detect the PostgreSQL cluster version and name.
        let clusters = xshell::cmd!(sh, "pg_lsclusters --no-header")
            .quiet()
            .read()
            .context("failed to list PostgreSQL clusters — is PostgreSQL installed?")?;

        let first_line = clusters
            .lines()
            .next()
            .context("no PostgreSQL clusters found")?;

        let mut parts = first_line.split_whitespace();
        let version = parts.next().context("could not parse cluster version")?;
        let name = parts.next().context("could not parse cluster name")?;

        // Configure trust authentication so passwordless connections work.
        Self::configure_trust_auth(sh, version, name)?;

        // Start the cluster (or restart if it was already running so config
        // changes take effect).
        tracing::info!("starting PostgreSQL cluster {version}/{name}");
        xshell::cmd!(sh, "pg_ctlcluster {version} {name} restart")
            .run()
            .with_context(|| {
                format!("failed to start PostgreSQL cluster {version}/{name}")
            })?;

        Ok(())
    }

    /// Rewrite pg_hba.conf to use trust authentication for local and TCP
    /// connections. This mirrors the `POSTGRES_HOST_AUTH_METHOD=trust` setting
    /// used by the Docker-based `xtask dev` command.
    fn configure_trust_auth(
        _sh: &xshell::Shell,
        version: &str,
        name: &str,
    ) -> anyhow::Result<()> {
        let hba_path = format!("/etc/postgresql/{version}/{name}/pg_hba.conf");

        let contents = std::fs::read_to_string(&hba_path)
            .with_context(|| format!("failed to read {hba_path}"))?;

        // Check if we already configured trust auth.
        if contents.contains("# durable-xtask: trust auth configured") {
            tracing::info!("trust authentication already configured");
            return Ok(());
        }

        tracing::info!("configuring trust authentication in {hba_path}");

        // Replace all auth methods with trust for local and host connections.
        let new_contents = format!(
            "\
# durable-xtask: trust auth configured
# This file was modified by `cargo xtask claude setup` to allow passwordless
# local development connections.
local   all             all                                     trust
host    all             all             127.0.0.1/32            trust
host    all             all             ::1/128                 trust
"
        );

        std::fs::write(&hba_path, new_contents)
            .with_context(|| format!("failed to write {hba_path}"))?;

        // Also ensure the postgres listen_addresses includes localhost.
        let conf_path = format!("/etc/postgresql/{version}/{name}/postgresql.conf");
        let conf = std::fs::read_to_string(&conf_path).unwrap_or_default();
        if !conf.contains("listen_addresses = '*'")
            && !conf.contains("listen_addresses = 'localhost'")
        {
            // Append listen_addresses if not already set to something useful.
            let mut conf = conf;
            conf.push_str("\nlisten_addresses = 'localhost'\n");
            let _ = std::fs::write(&conf_path, conf);
        }

        Ok(())
    }

    async fn connect(&self) -> anyhow::Result<sqlx::PgPool> {
        let options = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .username("postgres")
            .application_name("durable-xtask");

        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await;

        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

        loop {
            interval.tick().await;

            if tokio::time::Instant::now() > deadline {
                anyhow::bail!("failed to connect to PostgreSQL after 30s");
            }

            match sqlx::PgPool::connect_with(options.clone()).await {
                Ok(pool) => return Ok(pool),
                Err(e) => {
                    tracing::warn!("unable to connect to database: {e}");
                }
            }
        }
    }

    async fn migrate(pool: &sqlx::PgPool, migrator: &Migrator) -> anyhow::Result<()> {
        let table = Table::new("durable", "migrations");
        let options = Options {
            target: Target::Latest,
            transaction_mode: TransactionMode::Individual,
            migration_table: table,
            ..Default::default()
        };

        let mut conn = pool
            .acquire()
            .await
            .context("failed to get a connection to the database")?;

        tracing::info!("running database migrations");
        migrator.run(&mut conn, &options).await?;

        conn.detach();
        Ok(())
    }
}
