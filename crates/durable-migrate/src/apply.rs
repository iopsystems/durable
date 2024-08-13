use std::collections::BTreeMap;

use sqlx::postgres::PgRow;
use sqlx::{Connection, Row};

use crate::error::{DivergingMigrationError, ErrorData};
use crate::{Error, Migrator, Options, Table, Target, TransactionMode};

struct DatabaseMigration {
    version: i64,
    name: String,
    revert: Option<String>,
}

enum Operation<'a> {
    Apply {
        version: i64,
        name: &'a str,
        sql: &'a str,
        revert: Option<&'a str>,
    },
    Revert {
        version: i64,
        revert: &'a str,
        name: &'a str,
    },
}

impl<'a> Operation<'a> {
    pub fn is_revert(&self) -> bool {
        matches!(self, Self::Revert { .. })
    }
}

impl Migrator {
    async fn setup(&self, conn: &mut sqlx::PgConnection, options: &Options) -> Result<(), Error> {
        if let Some(schema) = options.migration_table.schema.as_deref() {
            sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {schema:?}"))
                .execute(&mut *conn)
                .await?;
        }

        #[rustfmt::skip]
        let query = format!(
            "\
            CREATE TABLE IF NOT EXISTS {table}(\
                version     bigint  NOT NULL PRIMARY KEY CHECK((version >= 0)),\
                name        text    NOT NULL,\
                revert      text\
            )\
            ",
            table = options.migration_table.as_sql()
        );
        sqlx::query(&query).execute(&mut *conn).await?;

        Ok(())
    }

    async fn applied_migrations(
        &self,
        conn: &mut sqlx::PgConnection,
        options: &Options,
    ) -> Result<Vec<DatabaseMigration>, Error> {
        let query = format!(
            "SELECT version, name, revert FROM {table} ORDER BY version ASC",
            table = options.migration_table.as_sql()
        );

        let migrations = sqlx::query(&query)
            .try_map(|record: PgRow| {
                Ok(DatabaseMigration {
                    version: record.get::<i64, _>("version"),
                    name: record.get("name"),
                    revert: record.get("revert"),
                })
            })
            .fetch_all(&mut *conn)
            .await?;

        Ok(migrations)
    }

    fn operations<'a>(
        &'a self,
        applied: &'a [DatabaseMigration],
        options: &Options,
    ) -> Result<Vec<Operation<'a>>, Error> {
        let known = BTreeMap::from_iter(self.migrations.iter().map(|m| (m.version as i64, m)));
        let applied = BTreeMap::from_iter(applied.iter().map(|m| (m.version, m)));
        let target = match options.target {
            Target::Latest => i64::MAX,
            Target::Version(version) => {
                i64::try_from(version).map_err(ErrorData::VersionOutOfRange)?
            }
        };

        if applied.is_empty() {
            let result = known
                .range(..=target)
                .map(|(_, migration)| Operation::Apply {
                    version: migration.version as i64,
                    name: &migration.name,
                    sql: &migration.sql,
                    revert: migration.revert.as_deref(),
                })
                .collect();

            return Ok(result);
        }

        if known.is_empty() {
            let mut operations = Vec::with_capacity(applied.len());

            for m in applied.values().rev() {
                operations.push(Operation::Revert {
                    version: m.version as i64,
                    name: &m.name,
                    revert: if options.prefer_local_revert {
                        known
                            .get(&m.version)
                            .and_then(|m| m.revert.as_deref())
                            .or_else(|| m.revert.as_deref())
                    } else {
                        m.revert
                            .as_deref()
                            .or_else(|| known.get(&m.version).and_then(|m| m.revert.as_deref()))
                    }
                    .ok_or(ErrorData::MissingDownMigration {
                        version: m.version as _,
                        name: m.name.clone(),
                    })?,
                });
            }

            return Ok(operations);
        }

        let divergence = known
            .values()
            .zip(applied.values())
            .find(|(known, applied)| {
                known.version as i64 != applied.version || known.name != applied.name
            });

        if let Some((known, applied)) = divergence {
            return Err(ErrorData::DivergingMigrations(DivergingMigrationError {
                expected_version: known.version,
                expected_name: known.name.clone(),
                found_version: applied.version as u64,
                found_name: applied.name.clone(),
            }))?;
        }

        let target = match options.target {
            Target::Latest => {
                // Since we want to go to the latest version, don't undo applied migrations.
                if applied.len() >= known.len() {
                    return Ok(Vec::new());
                }

                let operations = known
                    .values()
                    .skip(applied.len())
                    .map(|m| Operation::Apply {
                        version: m
                            .version
                            .try_into()
                            .expect("migration version out of range"),
                        name: &m.name,
                        sql: &m.sql,
                        revert: m.revert.as_deref(),
                    })
                    .collect();

                return Ok(operations);
            }
            Target::Version(target) => target,
        };
        let target: i64 = target
            .try_into()
            .map_err(|e| ErrorData::VersionOutOfRange(e))?;

        if applied.contains_key(&target) {
            let operations = applied
                .values()
                .rev()
                .take_while(|m| m.version > target)
                .map(|m| {
                    Ok(Operation::Revert {
                        version: m.version,
                        name: &m.name,
                        revert: if options.prefer_local_revert {
                            known
                                .get(&m.version)
                                .and_then(|m| m.revert.as_deref())
                                .or_else(|| m.revert.as_deref())
                        } else {
                            m.revert
                                .as_deref()
                                .or_else(|| known.get(&m.version).and_then(|m| m.revert.as_deref()))
                        }
                        .ok_or(ErrorData::MissingDownMigration {
                            version: m.version as _,
                            name: m.name.clone(),
                        })?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;

            Ok(operations)
        } else if known.contains_key(&target) {
            let operations = known
                .range(..=target)
                .filter(|(version, _)| !applied.contains_key(version))
                .map(|(&version, m)| Operation::Apply {
                    version,
                    name: &m.name,
                    sql: &m.sql,
                    revert: m.revert.as_deref(),
                })
                .collect();

            Ok(operations)
        } else {
            Err(ErrorData::MissingTargetMigration(target as u64).into())
        }
    }

    pub async fn run(&self, conn: &mut sqlx::PgConnection, options: &Options) -> Result<(), Error> {
        let mut tx = None;

        let conn = if options.dry_run || options.transaction_mode == TransactionMode::Single {
            &mut **tx.insert(conn.begin().await?)
        } else {
            &mut *conn
        };

        self.setup(&mut *conn, options).await?;
        let applied = self.applied_migrations(&mut *conn, options).await?;
        let operations = self.operations(&applied, options)?;

        if !options.allow_revert {
            let has_reverts = operations.iter().any(|op| op.is_revert());

            if has_reverts {
                return Err(ErrorData::WouldRevert.into());
            }
        }

        for operation in &operations {
            let mut tx = conn.begin().await?;

            let result = async {
                match operation {
                    &Operation::Apply {
                        version,
                        name,
                        sql,
                        revert,
                    } => {
                        tracing::debug!("running migration {version} - {name}");

                        sqlx::raw_sql(sql).execute(&mut *tx).await?;

                        let query = format!(
                            "INSERT INTO {table}(version, name, revert) VALUES ($1, $2, $3) ",
                            table = options.migration_table.as_sql()
                        );
                        sqlx::query(&query)
                            .bind(version)
                            .bind(name)
                            .bind(revert)
                            .execute(&mut *tx)
                            .await?;
                    }
                    &Operation::Revert {
                        version,
                        revert,
                        name,
                    } => {
                        tracing::debug!("reverting migration {version} - {name}");

                        let query = format!(
                            "DELETE FROM {table} WHERE version = $1 RETURNING version",
                            table = options.migration_table.as_sql()
                        );
                        sqlx::query(&query)
                            .bind(version)
                            .fetch_one(&mut *tx)
                            .await?;

                        sqlx::raw_sql(revert).execute(&mut *tx).await?;
                    }
                }

                Ok::<_, Error>(())
            }
            .await;

            if result.is_err() {
                tx.rollback().await?;
            } else {
                tx.commit().await?;
            }

            result?;
        }

        if operations.is_empty() {
            tracing::debug!("database schema is up to date!");
        }

        if let Some(tx) = tx {
            if options.dry_run {
                tx.rollback().await?;
            } else {
                tx.commit().await?;
            }
        }

        Ok(())
    }

    pub async fn read_database_version(
        &self,
        conn: &mut sqlx::PgConnection,
        migration_table: &Table,
    ) -> Result<Option<u64>, Error> {
        let options = Options {
            migration_table: migration_table.clone(),
            allow_revert: false,
            dry_run: true,
            prefer_local_revert: false,
            target: Target::Latest,
            transaction_mode: TransactionMode::Single,
        };

        let applied = self.applied_migrations(&mut *conn, &options).await?;

        // Emit an error if our migrations are invalid or the diverge from those in the
        // database.
        let _ = self.operations(&applied, &options)?;

        Ok(applied.last().map(|migration| migration.version as u64))
    }
}
