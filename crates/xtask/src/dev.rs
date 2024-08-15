use std::mem::ManuallyDrop;
use std::time::Duration;

use anyhow::Context;
use durable_migrate::{Migrator, Options, Table, Target, TransactionMode};
use tokio::process::Child;

#[derive(Debug, clap::Args)]
pub struct Dev {
    /// Disallow overwriting a pre-existing environment file.
    #[arg(long)]
    no_overwrite_env: bool,

    /// Start the database server in the background and then return immediately
    /// after having run migrations.
    #[arg(long, short = 'd')]
    detach: bool,
}

impl Dev {
    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) -> anyhow::Result<()> {
        self._run().await
    }

    async fn _run(self) -> anyhow::Result<()> {
        use tokio::process::Command;

        let root = crate::workspace_root()?;
        let migrator = Migrator::from_dir(root.join("crates/durable-runtime/migrations"))?;

        Self::assert_not_running()?;

        let mut command = Command::new("docker");
        command.arg("run");
        command.arg("-p");
        command.arg("5432:5432");
        command.arg("-e");
        command.arg("POSTGRES_HOST_AUTH_METHOD=trust");
        command.arg("--rm");
        command.arg("--name");
        command.arg("durable-postgres");

        if self.detach {
            command.arg("-d");
        }

        command.arg("postgres:15");
        command.kill_on_drop(true);

        let mut child = command
            .spawn()
            .context("failed to spawn `docker run postgres:15`")?;

        let pool = tokio::select! {
            biased;

            _ = child.wait(), if !self.detach => None,
            _ = tokio::signal::ctrl_c() => None,
            res = Self::connect() => {
                if res.is_err() {
                    signal(&child);
                    let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
                }

                Some(res?)
            },
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                signal(&child);
                let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;

                anyhow::bail!("failed to connect to the newly started database after 60s")
            }
        };

        if let Some(pool) = pool {
            if let Err(e) = Self::migrate(&pool, &migrator).await {
                println!("warning: migrating the database failed with an error:\n {e:?}");
            }

            pool.close().await
        }

        let env = root.join(".env");
        let exists = env.exists();

        let _guard = if !exists || !self.no_overwrite_env {
            std::fs::write(
                &env,
                b"DATABASE_URL=postgres://postgres@localhost:5432/postgres\n",
            )
            .context("failed to write the .env file")?;

            if self.detach {
                None
            } else {
                Some(Defer::new(|| {
                    let _ = std::fs::remove_file(&env);
                }))
            }
        } else {
            if exists && self.no_overwrite_env {
                tracing::warn!("refusing to overwrite existing .env file");
            }

            None
        };

        let status = child
            .wait()
            .await
            .context("failed to wait for the docker process to complete")?;

        match status.code() {
            Some(0) => (),
            Some(n) => anyhow::bail!("`docker run ... postgres:15` exited with exit status {n}"),
            None => anyhow::bail!("`docker run ... postgres:15` exited abnormally"),
        }

        Ok(())
    }

    async fn connect() -> anyhow::Result<sqlx::PgPool> {
        let options = sqlx::postgres::PgConnectOptions::new()
            .host("localhost")
            .username("postgres")
            .application_name("durable-xtask");
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        // Skip the first tick since there is no chance of the database being ready yet.
        interval.tick().await;

        loop {
            interval.tick().await;

            let error = match sqlx::PgPool::connect_with(options.clone()).await {
                Ok(pool) => return Ok(pool),
                Err(e) => e,
            };

            tracing::warn!("unable to connect to database: {error}");
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

    fn assert_not_running() -> anyhow::Result<()> {
        let sh = xshell::Shell::new()?;

        let exists = xshell::cmd!(sh, "docker container inspect durable-postgres")
            .ignore_stdout()
            .quiet()
            .run()
            .is_ok();

        if exists {
            anyhow::bail!(
                "there is already a container with the name `durable-postgres` running, kill that \
                 one before starting a new one"
            );
        }

        Ok(())
    }
}

#[cfg(unix)]
fn signal(child: &Child) {
    let Some(id) = child.id() else { return };

    unsafe { libc::kill(id as i32, libc::SIGINT) };
}

#[cfg(not(unix))]
fn signal(child: &Child) {
    child.kill();
}

struct Defer<F: FnOnce()>(ManuallyDrop<F>);

impl<F: FnOnce()> Defer<F> {
    pub fn new(func: F) -> Self {
        Self(ManuallyDrop::new(func))
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        // SAFETY: We have sole ownership over self and nothing else will be touching it
        //         so removing `func` is safe.
        let func = unsafe { ManuallyDrop::take(&mut self.0) };

        func()
    }
}
