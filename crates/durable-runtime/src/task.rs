use std::any::Any;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::value::RawValue;
use sqlx::types::Json;
use sqlx::PgConnection;
use tokio::sync::broadcast::Receiver;

use crate::error::TaskStatus;
use crate::event::Notification;
use crate::util::AsyncFnOnce;
use crate::worker::{SharedState, TaskData};
use crate::Config;

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

/// A running workflow transaction.
pub struct Transaction {
    label: String,
    index: i32,

    // SAFETY NOTE:
    // When Some this strema contains a reference to conn. It must be cleared before it is safe to
    // access conn.
    stream: Option<QueryStream<'static>>,

    // We box this field so that it has a stable address for `stream` to refer to.
    conn: Option<Box<sqlx::Transaction<'static, sqlx::Postgres>>>,

    /// The log messages emitted during this transaction.
    ///
    /// These will be committed to the database at the end of the transaction.
    pub logs: String,

    /// Kept for convenience on some methods.
    shared: Arc<SharedState>,
}

impl Transaction {
    fn new(label: String, index: i32, shared: Arc<SharedState>) -> Self {
        Self {
            label,
            index,
            stream: None,
            conn: None,
            logs: String::new(),
            shared,
        }
    }

    /// Access the label for the current transaction.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Get the index of this transaction within the workflow.
    pub fn index(&self) -> i32 {
        self.index
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

    /// Attach a new database connection to this transaction.
    ///
    /// This returns an error if this transaction already has a database
    /// transaction attached to it. This will also unconditionally reset any
    /// existing query output stream in this transaction.
    pub fn set_conn(
        &mut self,
        txn: sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> anyhow::Result<()> {
        self.stream = None;
        if self.conn.is_some() {
            anyhow::bail!("transaction already had a database connection associated with it");
        }

        self.conn = Some(Box::new(txn));
        Ok(())
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

    /// Write some logs out to the transaction log field.
    ///
    /// This method will automatically take care of truncating the logs if they
    /// are over the limit.
    pub fn write_logs(&mut self, message: &str) {
        let remaining = self
            .shared
            .config
            .max_log_bytes_per_transaction
            .saturating_sub(self.logs.len());
        let truncated = truncate_to_prev_char_boundary(message, remaining);

        self.logs.push_str(truncated);
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

    /// Subscribe to notification events.
    pub fn subscribe_notifications(&self) -> Receiver<Notification> {
        self.shared.notifications.subscribe()
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
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
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
    pub async fn enter<T>(&mut self, options: TransactionOptions) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(txn) = self.transaction() {
            anyhow::bail!(
                "attempted to start transaction {:?} while already within transaction {:?}",
                options.label,
                txn.label
            );
        }

        let mut tx = self.pool().begin().await?;
        let record = sqlx::query!(
            r#"
            SELECT
                label,
                value as "value: Json<Box<RawValue>>"
             FROM durable.event
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

            if record.label != options.label {
                anyhow::bail!(
                    "workflow execution is non-deterministic: stored event at index {} has label \
                     {:?} but the workflow requested {:?}",
                    self.txn_index,
                    record.label,
                    options.label
                );
            }

            self.txn_index += 1;
            let value: T = serde_json::from_str(record.value.get()).with_context(|| {
                format!(
                    "internal error: failed to deserialize internal event data of type `{}`",
                    std::any::type_name::<T>()
                )
            })?;

            Ok(Some(value))
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

            tracing::trace!(
                index = self.txn_index,
                "entering transaction {}",
                options.label
            );

            let mut txn = Transaction::new(options.label, self.txn_index, self.shared.clone());
            if options.database {
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

        let running_on = sqlx::query!(
            "SELECT running_on FROM durable.task WHERE id = $1",
            self.task_id()
        )
        .fetch_one(&mut *tx)
        .await?
        .running_on;
        if running_on != Some(self.worker_id) {
            // This task is no longer running on the current worker. Don't commit anything,
            // and abort the task.
            return Err(anyhow::Error::new(TaskStatus::NotScheduledOnWorker));
        }

        // tracing::info!(
        //     label = txn.label,
        //     "completing transaction {}",
        //     self.txn_index
        // );

        sqlx::query!(
            r#"
            INSERT INTO durable.event(task_id, index, label, value)
            VALUES ($1, $2, $3, $4)
            "#,
            self.task_id(),
            self.txn_index,
            txn.label,
            Json(data) as Json<&T>
        )
        .execute(&mut *tx)
        .await
        .with_context(|| {
            format!(
                "failed to insert event {} with label {}",
                self.txn_index, txn.label
            )
        })?;

        if !txn.logs.is_empty() {
            tracing::debug!(
                target: "durable::task_log",
                "task {}: {}",
                self.task_id(),
                txn.logs.trim_end()
            );

            sqlx::query!(
                r#"
                INSERT INTO durable.log(task_id, index, message)
                VALUES ($1, $2, $3)
                "#,
                self.task_id(),
                self.txn_index,
                std::mem::take(&mut txn.logs)
            )
            .execute(&mut *tx)
            .await?;
        }

        tracing::trace!(index = self.txn_index, "exiting transaction {}", txn.label);

        tx.commit().await?;
        self.txn_index += 1;

        Ok(())
    }

    /// Mark the current transaction as suspended.
    ///
    /// This function will use the provided connection to do so. If the
    /// connection is part of a transaction you can ensure that other state has
    /// not been modified before committing the transaction.
    ///
    /// On success, returns a [`TaskState`] object that should be returned as an
    /// error.
    pub async fn suspend(
        &mut self,
        conn: &mut PgConnection,
        timeout: Option<DateTime<Utc>>,
    ) -> anyhow::Result<TaskStatus> {
        sqlx::query!(
            "UPDATE durable.task
            SET state = 'suspended',
                running_on = NULL,
                wakeup_at = $2
            WHERE id = $1",
            self.task_id(),
            timeout
        )
        .execute(&mut *conn)
        .await?;

        Ok(TaskStatus::Suspend)
    }

    pub async fn do_transaction<F, T>(
        &mut self,
        options: TransactionOptions,
        func: F,
    ) -> anyhow::Result<T>
    where
        F: for<'t> FnOnce(&'t mut Self) -> BoxFuture<anyhow::Result<T>> + Send + 'static,
        T: Serialize + DeserializeOwned + Send,
    {
        if let Some(data) = self.enter(options).await? {
            return Ok(data);
        }

        let data = match func.call((&mut *self,)).await {
            Ok(data) => data,
            Err(e) => {
                let mut txn = match self.txn.take() {
                    Some(txn) => txn,
                    None => anyhow::bail!(
                        "internal error: transaction removed the transaction data within the task \
                         context"
                    ),
                };

                if let Some(tx) = txn.take_conn() {
                    let _ = tx.rollback().await;
                }

                return Err(e);
            }
        };

        self.exit(&data).await?;
        Ok(data)
    }

    pub async fn maybe_do_transaction<F, T>(
        &mut self,
        options: TransactionOptions,
        func: F,
    ) -> anyhow::Result<T>
    where
        F: for<'t> FnOnce(&'t mut Transaction) -> BoxFuture<anyhow::Result<T>> + Send + 'static,
        T: Serialize + DeserializeOwned + Send,
    {
        if let Some(txn) = self.transaction_mut() {
            return func.call((txn,)).await;
        }

        self.do_transaction(options, |task: &mut Self| {
            Box::pin(async move {
                let txn = task.transaction_mut().expect("task had no transaction");
                func.call((txn,)).await
            })
        })
        .await
    }

    pub async fn maybe_do_transaction_sync<F, T>(
        &mut self,
        options: TransactionOptions,
        func: F,
    ) -> anyhow::Result<T>
    where
        F: FnOnce(&mut Transaction) -> anyhow::Result<T> + Send + 'static,
        T: Serialize + DeserializeOwned + Send,
    {
        self.maybe_do_transaction(options, |txn| Box::pin(async move { func(txn) }))
            .await
    }
}

fn truncate_to_prev_char_boundary(s: &str, len: usize) -> &str {
    if len >= s.len() {
        return s;
    }

    let lower_bound = len.saturating_sub(3);
    let bytes = s.as_bytes();

    let new_index = bytes[lower_bound..=len]
        .iter()
        .copied()
        .rposition(|b| b < 128 || b >= 192)
        .unwrap();

    &s[..new_index]
}

impl Deref for Task {
    type Target = TaskState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Task {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}
