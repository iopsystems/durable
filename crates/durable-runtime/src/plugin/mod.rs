use std::any::Any;
use std::sync::Arc;

use futures_util::stream::BoxStream;
use serde::Serialize;
use serde_json::value::RawValue;
use sqlx::types::Json;
use wasmtime::component::Linker;

use crate::error::AbortError;
use crate::worker::{SharedState, TaskData};
use crate::Config;

pub mod durable;

pub type QueryStream<'a> =
    BoxStream<'a, Result<sqlx::Either<QueryResult, sqlx::postgres::PgRow>, sqlx::Error>>;

#[derive(Copy, Clone, Debug, Default)]
pub struct QueryResult {
    pub rows_affected: u64,
}

impl From<sqlx::postgres::PgQueryResult> for QueryResult {
    fn from(value: sqlx::postgres::PgQueryResult) -> Self {
        Self {
            rows_affected: value.rows_affected(),
        }
    }
}

impl Extend<Self> for QueryResult {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        self.rows_affected += iter.into_iter().map(|r| r.rows_affected).sum::<u64>();
    }
}

/// A plugin for the durable runtime.
///
/// Plugins allow you to expose custom functions to workers running within the
/// WASM vm. This trait provides the hooks necessary for a plugin to set itself
/// up when a task is started.
pub trait Plugin: Send + Sync {
    /// The name of this plugin.
    ///
    /// This is used for error messages in case something goes wrong when
    /// setting up the plugin.
    fn name(&self) -> &str;

    /// Perform setup required by this plugin.
    ///
    /// This should add any functions exported by this plugin to the linker and
    /// setup any state this plugin needs within the task plugin data.
    fn setup(&self, linker: &mut Linker<Task>, store: &mut Task) -> wasmtime::Result<()>;
}

/// A running workflow transaction.
pub struct Transaction {
    label: String,

    // SAFETY NOTE:
    // When Some this strema contains a reference to conn. It must be cleared before it is safe to
    // access conn.
    stream: Option<QueryStream<'static>>,

    // We box this field so that it has a stable address for `stream` to refer to.
    conn: Option<Box<sqlx::Transaction<'static, sqlx::Postgres>>>,
}

impl Transaction {
    fn new(label: String) -> Self {
        Self {
            label,
            stream: None,
            conn: None,
        }
    }

    /// Access the label for the current transaction.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Access the database connection associated with this transaction.
    ///
    /// This will only exist if a database transaction was requested when the
    /// workflow transaction was entered.
    ///
    /// Note that calling this method will drop any in-progress query stream
    /// within the transaction.
    pub fn conn(&mut self) -> Option<&mut sqlx::Transaction<'static, sqlx::Postgres>> {
        // Clear the current query stream before accessing `conn`.
        self.stream = None;
        self.conn.as_deref_mut()
    }

    /// Take the database connection associated with this transaction.
    ///
    /// This will only exist if a database transaction was requested when the
    /// workflow transaction was entered.
    ///
    /// Note that calling this method will drop any in-progress query stream
    /// within the transaction.
    pub fn take_conn(&mut self) -> Option<sqlx::Transaction<'static, sqlx::Postgres>> {
        self.stream = None;
        self.conn.take().map(|c| *c)
    }

    /// Access the output stream for the last query run within this transaction.
    pub fn stream<'t>(&'t mut self) -> Option<&'t mut QueryStream<'t>> {
        let stream = self.stream.as_mut()?;

        // SAFETY: The actual lifetime of the stream is 't. This is just changing it
        //         back to match reality instead of the 'static we need for the
        //         self-reference to be allowed.
        let stream: &'t mut QueryStream<'t> =
            unsafe { &mut *(stream as *mut QueryStream as *mut QueryStream<'t>) };

        Some(stream)
    }

    /// Take the output stream for the last query run within this transaction.
    pub fn take_stream<'t>(&'t mut self) -> Option<QueryStream<'t>> {
        self.stream.take()
    }

    /// Start a new query within this database transaction and store the stream
    /// of its results within the transaction for future use.
    ///
    /// This method is meant to be used when the results of the query need to be
    /// iterated over by the wasm module. If you just want to make a query in a
    /// plugin you can use sqlx directly.
    pub fn start_query<F>(&mut self, func: F) -> anyhow::Result<()>
    where
        F: for<'t> FnOnce(&'t mut sqlx::Transaction<'static, sqlx::Postgres>) -> QueryStream<'t>,
    {
        let Some(conn) = self.conn() else {
            anyhow::bail!("no database connection associated with the current transaction")
        };

        let stream: QueryStream = func(conn);

        // SAFETY: We ensure that self.stream does not outlive the transaction it was
        //         created from.
        let stream: QueryStream<'static> = unsafe { std::mem::transmute(stream) };
        self.stream = Some(stream);

        Ok(())
    }
}

pub struct Task {
    pub state: TaskState,
    pub plugins: anymap3::Map<dyn Any + Send>,
}

pub struct TaskState {
    shared: Arc<SharedState>,
    worker_id: i64,

    task: TaskData,

    txn_index: i32,
    txn: Option<Transaction>,
}

impl TaskState {
    pub(crate) fn new(shared: Arc<SharedState>, task: TaskData, worker_id: i64) -> Self {
        Self {
            shared,
            task,
            worker_id,
            txn_index: 0,
            txn: None,
        }
    }
}

impl TaskState {
    /// Get the id of the current task.
    pub fn task_id(&self) -> i64 {
        self.task.id
    }

    /// Get the name of the current task.
    pub fn task_name(&self) -> &str {
        &self.task.name
    }

    /// Get the JSON data associated with the current task.
    pub fn task_data(&self) -> &RawValue {
        &self.task.data
    }

    /// Get the worker id that we are currently runing on.
    ///
    /// Note that it is not safe to expose this to the workflow outside of a
    /// transaction because the worker that the workflow is running on may
    /// change out from underneath it.
    pub fn worker_id(&self) -> i64 {
        self.worker_id
    }

    /// Access the database connection pool for the worker.
    pub fn pool(&self) -> &sqlx::PgPool {
        &self.shared.pool
    }

    /// Access the reqwest client for the worker.
    pub fn client(&self) -> &reqwest::Client {
        &self.shared.client
    }

    /// Access the durable runtime configuration.
    pub fn config(&self) -> &Config {
        &self.shared.config
    }

    /// Access the current transaction.
    pub fn transaction(&self) -> Option<&Transaction> {
        self.txn.as_ref()
    }

    /// Access the current transaction mutably.
    pub fn transaction_mut(&mut self) -> Option<&mut Transaction> {
        self.txn.as_mut()
    }
}

#[derive(Clone, Debug)]
pub struct TransactionOptions {
    label: String,
    database: bool,
}

impl TransactionOptions {
    pub fn new(label: String) -> Self {
        Self {
            label,
            database: false,
        }
    }

    pub fn database(mut self, database: bool) -> Self {
        self.database = database;
        self
    }
}

impl TaskState {
    /// Assert that there is an active transaction right now and return said
    /// transaction.
    ///
    /// If this is run outside of a transaction then it will return an error so
    /// that the function can trap.
    pub fn assert_in_transaction(&mut self, operation: &str) -> anyhow::Result<&mut Transaction> {
        match self.transaction_mut() {
            Some(txn) => Ok(txn),
            None => {
                anyhow::bail!(
                    "attempted to run impure function `{operation}` outside of a transaction"
                )
            }
        }
    }

    /// Enter a new transaction.
    ///
    /// Returns the existing output data for this transaction, should the
    /// transaction already have been executed.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * Any of the database queries result in an error.
    /// * The transaction index exceeds [`Config::max_workflow_events`] and this
    ///   transaction is not already recorded in the database.
    /// * The requested label does not match the one recorded in the database.
    pub async fn enter(
        &mut self,
        opts: TransactionOptions,
    ) -> anyhow::Result<Option<Box<RawValue>>> {
        if let Some(txn) = self.transaction() {
            anyhow::bail!(
                "attempted to start transaction {:?} while already within transaction {:?}",
                opts.label,
                txn.label
            );
        }

        let mut tx = self.pool().begin().await?;
        let record = sqlx::query!(
            r#"
            SELECT
                label,
                value as "value: Json<Box<RawValue>>"
             FROM event
            WHERE task_id = $1
              AND index = $2
            "#,
            self.task_id(),
            self.txn_index
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(record) = record {
            tx.rollback().await?;

            if record.label != opts.label {
                anyhow::bail!(
                    "workflow execution is non-deterministic: stored event at index {} has label \
                     {:?} but the workflow requested {:?}",
                    self.txn_index,
                    record.label,
                    opts.label
                );
            }

            self.txn_index += 1;
            Ok(Some(record.value.0))
        } else {
            let max_txn_index = self
                .config()
                .max_workflow_events
                .try_into()
                .unwrap_or(i32::MAX);

            if self.txn_index >= max_txn_index {
                tx.rollback().await?;

                anyhow::bail!(
                    "workflow exceeded the configured maximum number of allowed workflow \
                     transactions (configured limit is {})",
                    max_txn_index
                );
            }

            let mut txn = Transaction::new(opts.label);
            if opts.database {
                txn.conn = Some(Box::new(tx));
            } else {
                tx.commit().await?;
            }

            self.txn = Some(txn);
            Ok(None)
        }
    }

    /// Exit the current transaction.
    ///
    /// # Errors
    /// This function will return an error if:
    /// * Any of the database queries within result in an error.
    /// * There is no current transaction.
    /// * The data could not be serialized to JSON.
    pub async fn exit<T>(&mut self, data: &T) -> anyhow::Result<()>
    where
        T: ?Sized + Serialize,
    {
        let mut txn = match self.txn.take() {
            Some(txn) => txn,
            None => anyhow::bail!("attempted to exit a transaction without having entered one"),
        };

        let mut tx = match txn.take_conn() {
            Some(mut tx) => {
                // Check if the transaction is not in an aborted state by running a query
                if sqlx::query("SELECT 1").execute(&mut *tx).await.is_ok() {
                    tx
                } else {
                    self.shared.pool.begin().await?
                }
            }
            _ => self.shared.pool.begin().await?,
        };

        let running_on = sqlx::query!("SELECT running_on FROM task WHERE id = $1", self.task_id())
            .fetch_one(&mut *tx)
            .await?
            .running_on;
        if running_on != Some(self.worker_id) {
            // This task is no longer running on the current worker. Don't commit anything,
            // and abort the task.
            return Err(anyhow::Error::new(AbortError));
        }

        sqlx::query!(
            r#"
            INSERT INTO event(task_id, index, label, value)
            VALUES ($1, $2, $3, $4)
            "#,
            self.task_id(),
            self.txn_index,
            txn.label,
            Json(data) as Json<&T>
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.txn_index += 1;

        Ok(())
    }
}
