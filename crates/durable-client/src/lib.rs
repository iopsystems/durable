use std::sync::{Arc, PoisonError, RwLock, Weak};

use chrono::{Duration, Utc};
use error::ErrorImpl;
use sha2::{Digest, Sha256};
use sqlx::types::Json;
use sqlx::Acquire;
use wasmparser::Validator;
use weak_table::weak_value_hash_map::Entry;
use weak_table::WeakValueHashMap;

use crate::program::{ProgramData, ProgramHash};

mod error;
pub mod event;
mod program;
mod task;
mod util;

pub use self::error::{DurableError, DurableErrorKind};
pub use self::program::{Program, ProgramOptions};
pub use self::task::{ExitStatus, Task, TaskState};

#[derive(Clone)]
pub struct DurableClient {
    pool: sqlx::PgPool,
    data: Arc<ClientData>,
}

struct ClientData {
    programs: RwLock<WeakValueHashMap<[u8; 32], Weak<ProgramData>>>,
}

impl DurableClient {
    /// Create a new durable client from a PgPool instance.
    pub fn new(pool: sqlx::PgPool) -> Result<Self, DurableError> {
        // At the moment this constructor is infallible. However, we return an error
        // here in case we want to validate that we are actually connecting to a
        // compatible database change.

        Ok(Self {
            pool,
            data: Arc::new(ClientData {
                programs: RwLock::new(WeakValueHashMap::new()),
            }),
        })
    }

    /// Load a new program for use by workflows.
    ///
    /// You can then use the resulting [`Program`] to launch workflows
    /// by calling [`launch`].
    ///
    /// Creating [`Program`]s is rather expensive. It is preferable to keep the
    /// [`Program`] instance around and reuse it instead of creating new ones
    /// all the time.
    ///
    /// # Errors
    /// This method returns errors in the following cases:
    /// * The WASM program fails to validate.
    /// * The WASM program is not a WASM component.
    /// * An error occurs while communicating with the database.
    ///
    /// [`launch`]: DurableClient::launch
    pub async fn program(&self, opts: ProgramOptions) -> Result<Program, DurableError> {
        // TODO: Should validation and hashing go in a blocking_spawn call?

        let mut validator = Validator::new_with_features(supported_wasm_features());
        validator
            .validate_all(&opts.wasm)
            .map_err(|e| DurableError(e.into()))?;

        if !wasmparser::Parser::is_component(&opts.wasm) {
            return Err(DurableError(ErrorImpl::ProgramIsNotAComponent));
        }

        let mut hasher = Sha256::new();
        hasher.update(&opts.wasm);
        let hash: ProgramHash = hasher.finalize().into();

        let mut conn = self.pool.acquire().await?;
        let data = ProgramData::register(hash, opts.wasm, opts.name, &mut conn).await?;
        drop(conn);

        let data = Arc::new(data);
        let mut programs = self
            .data
            .programs
            .write()
            .unwrap_or_else(PoisonError::into_inner);

        let data = match programs.entry(hash) {
            Entry::Vacant(entry) => entry.insert(data),
            Entry::Occupied(entry) => entry.get_strong(),
        };

        Ok(Program::new(data))
    }

    /// Launch a new workflow with the provided program and task data.
    pub async fn launch<T>(
        &self,
        name: impl AsRef<str>,
        program: &Program,
        data: &T,
    ) -> Result<Task, DurableError>
    where
        T: ?Sized + serde::Serialize,
    {
        self._launch(name.as_ref(), program, data).await
    }

    async fn _launch<T>(
        &self,
        name: &str,
        program: &Program,
        data: &T,
    ) -> Result<Task, DurableError>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut conn = self.pool.acquire().await?;
        self.launch_with(name, program, data, &mut conn).await
    }

    /// Launch a new workflow using the provided database connection.
    ///
    /// This allows program launches to be done as part of a larger transaction.
    pub async fn launch_with<T>(
        &self,
        name: &str,
        program: &Program,
        data: &T,
        conn: &mut sqlx::PgConnection,
    ) -> Result<Task, DurableError>
    where
        T: ?Sized + serde::Serialize,
    {
        let mut tx = conn.begin().await?;

        let now = Utc::now();
        let last_used = program.0.last_used.get();

        if last_used < now - Duration::hours(1) {
            let record = sqlx::query!(
                "UPDATE durable.wasm
                      SET last_used = CURRENT_TIMESTAMP
                    WHERE id = $1
                    RETURNING last_used",
                program.0.id()
            )
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(record) = record {
                program.0.last_used.set(record.last_used);
            }
        }

        let workflow = loop {
            // Create a savepoint so that we can rollback if something goes wrong here.
            let mut stx = tx.begin().await?;
            let result = sqlx::query!(
                "
                    INSERT INTO durable.task(name, wasm, data, running_on)
                    SELECT
                        $1 as name,
                        $2 as wasm,
                        $3 as data,
                        (SELECT id
                           FROM durable.worker
                          ORDER BY random()
                          LIMIT 1
                          FOR SHARE SKIP LOCKED
                        ) as running_on
                    RETURNING id
                    ",
                name,
                program.0.id(),
                Json(data) as Json<&T>
            )
            .fetch_one(&mut *stx)
            .await;

            let error = match result {
                Ok(record) => {
                    stx.commit().await?;
                    break Task { id: record.id };
                }
                Err(e) => e,
            };

            stx.rollback().await?;

            match error {
                sqlx::Error::Database(err) if err.is_foreign_key_violation() => {
                    match err.constraint() {
                        // The worker got deleted out from underneath us as we were running the
                        // query.
                        //
                        // This is unlikely but can be solved with a retry anyway.
                        Some("fk_worker") => continue,

                        // The WASM module we thought we had doesn't exist anymore.
                        //
                        // In this case we just need to recreate it in the database.
                        Some("fk_wasm") => {
                            program.0.reregister(&mut tx).await?;
                            continue;
                        }

                        _ => {
                            let _ = tx.rollback().await;
                            return Err(sqlx::Error::Database(err).into());
                        }
                    }
                }
                e => {
                    let _ = tx.rollback().await;
                    return Err(e.into());
                }
            }
        };

        tx.commit().await?;
        Ok(workflow)
    }
}

fn supported_wasm_features() -> wasmparser::WasmFeatures {
    use wasmparser::WasmFeatures;

    let mut features = WasmFeatures::default();

    // The durable-runtime does not support wasm threads since they are not
    // compatible with a single order of execution.
    features.remove(WasmFeatures::THREADS);

    features
}
