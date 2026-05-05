//! Database storage abstraction for the durable runtime.
//!
//! The [`Storage`] trait collects every query the runtime issues against the
//! durable schema. Each method corresponds to one logical operation and its
//! body is a verbatim move of the SQL that previously lived inline at the
//! call site. Transaction lifecycle (`pool.begin()`, `pool.acquire()`,
//! `commit`/`rollback`) remains at the call sites — methods take a borrowed
//! [`sqlx::PgConnection`], which works for both pooled connections and
//! transactions via sqlx's `Executor` blanket impl.
//!
//! The current implementation, [`PgStorage`], is Postgres-specific and is
//! intended to be the only impl for now. The trait exists so that future
//! work can introduce alternative backends (e.g. SQLite with a polling
//! executor) without further restructuring of the runtime.

mod pg;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::value::RawValue;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::PgQueryResult;
use sqlx::types::Json;
use sqlx::PgConnection;

pub(crate) use self::pg::PgStorage;
use crate::task::RecordedEvent;
use crate::worker::TaskData;

/// The state of a row in `durable.task`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "durable.task_state", rename_all = "lowercase")]
pub(crate) enum TaskState {
    Ready,
    Active,
    Suspended,
    Complete,
    Failed,
}

/// A worker row, as returned by the validate-workers sequencing query.
#[derive(Debug)]
pub(crate) struct WorkerRecord {
    pub id: i64,
    pub heartbeat_at: DateTime<Utc>,
}

/// A row from `durable.event`, as returned when looking up a previously
/// recorded transaction by index.
#[derive(Debug)]
pub(crate) struct StoredEvent {
    pub label: String,
    pub value: Json<Box<RawValue>>,
}

/// A row from `durable.notification`, as returned by `poll_notification`.
#[derive(Debug)]
pub(crate) struct PolledNotification {
    pub created_at: DateTime<Utc>,
    pub event: String,
    pub data: Json<Box<RawValue>>,
}

/// Database operations issued by the durable runtime.
///
/// All methods accept a borrowed [`PgConnection`] and leave transaction
/// lifecycle to the caller. See module-level docs for the rationale.
#[async_trait]
pub(crate) trait Storage: Send + Sync + 'static {
    /// Access the underlying connection pool.
    ///
    /// Used by callers that need to acquire a connection or open a
    /// transaction directly. Currently unused — transaction lifecycle is
    /// driven through `SharedState::pool` until phase 2 of the storage
    /// abstraction.
    #[allow(dead_code)]
    fn pool(&self) -> &sqlx::PgPool;

    // ---------------------------------------------------------------------
    // Worker lifecycle
    // ---------------------------------------------------------------------

    /// Insert a new worker row and return its id.
    async fn insert_worker(&self, conn: &mut PgConnection) -> sqlx::Result<i64>;

    /// Delete the worker row with the given id.
    async fn delete_worker(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Refresh the heartbeat for the given worker. Returns `true` if the row
    /// still exists.
    async fn heartbeat_worker(&self, conn: &mut PgConnection, worker_id: i64)
        -> sqlx::Result<bool>;

    /// Delete the worker we are following if its heartbeat has expired.
    async fn delete_following_expired_worker(
        &self,
        conn: &mut PgConnection,
        following: i64,
        timeout: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    /// Delete every other expired worker, leaving the current one alone.
    async fn delete_other_expired_workers(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        timeout: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    /// Return the next worker in the validation sequence relative to
    /// `worker_id`. See `Worker::validate_workers` for the algorithm.
    async fn next_worker_in_sequence(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<Option<WorkerRecord>>;

    /// Return the id of the current cluster leader (the oldest worker), if
    /// any.
    async fn load_leader_id(&self, conn: &mut PgConnection) -> sqlx::Result<Option<i64>>;

    // ---------------------------------------------------------------------
    // Task scheduling
    // ---------------------------------------------------------------------

    /// Wake any suspended tasks whose `wakeup_at` has elapsed past the
    /// configured suspend margin.
    async fn wake_suspended_tasks(
        &self,
        conn: &mut PgConnection,
        suspend_margin: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    /// Return the earliest `wakeup_at` of any currently suspended task.
    async fn next_wakeup_at(&self, conn: &mut PgConnection) -> sqlx::Result<Option<DateTime<Utc>>>;

    /// Delete a batch of completed tasks older than `cleanup_age`.
    async fn cleanup_old_tasks(
        &self,
        conn: &mut PgConnection,
        cleanup_age: PgInterval,
        limit: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Force-resume any tasks stuck waiting on notifications older than 10
    /// minutes.
    async fn unwedge_stuck_notifications(
        &self,
        conn: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult>;

    /// Reset a batch of failed tasks back to the `ready` state.
    async fn reset_failed_tasks(
        &self,
        conn: &mut PgConnection,
        failed_ids: &[i64],
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Atomically claim up to `allowed` new tasks for `worker_id`.
    async fn claim_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        allowed: i64,
    ) -> sqlx::Result<Vec<TaskData>>;

    /// Release any ready tasks currently assigned to `worker_id`. Used when
    /// a worker hits its capacity before committing a claim.
    async fn release_owned_ready_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Reset a single task back to `ready` so it can be picked up again,
    /// only if it is still owned by the given worker.
    async fn reset_task_for_retry(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Mark a task as successfully completed.
    async fn mark_task_complete(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Mark a task as failed.
    async fn mark_task_failed(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    // ---------------------------------------------------------------------
    // Task execution
    // ---------------------------------------------------------------------

    /// Fetch the wasm bytecode for a stored program.
    async fn fetch_wasm_blob(&self, conn: &mut PgConnection, wasm_id: i64)
        -> sqlx::Result<Vec<u8>>;

    /// Load up to 1000 recorded events for a task, used for replay.
    async fn fetch_recorded_events(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Vec<RecordedEvent>>;

    /// Look up a single recorded event for a task by its index.
    async fn fetch_event_at_index(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
    ) -> sqlx::Result<Option<StoredEvent>>;

    /// Insert an event row, optionally insert a log row, and return the
    /// task's `running_on` value — all in one round-trip.
    async fn commit_event_with_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        label: &str,
        value: Json<&RawValue>,
        message: Option<&str>,
    ) -> sqlx::Result<Option<i64>>;

    /// Suspend a task and set an optional wakeup timestamp.
    async fn suspend_task(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        wakeup_at: Option<DateTime<Utc>>,
    ) -> sqlx::Result<PgQueryResult>;

    /// Suspend a task without touching `wakeup_at`. Used by the notification
    /// blocking path, which must not clobber an existing wakeup deadline.
    async fn suspend_task_no_wakeup(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    // ---------------------------------------------------------------------
    // Logs
    // ---------------------------------------------------------------------

    /// Append a row to `durable.log`.
    async fn insert_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult>;

    /// Insert a log row, replacing the message if a row already exists at
    /// the same `(task_id, index)`. Used to record the final error message
    /// for a task.
    async fn upsert_log_error(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult>;

    // ---------------------------------------------------------------------
    // Notifications
    // ---------------------------------------------------------------------

    /// Pop the oldest notification for a task, with row-level locking.
    async fn poll_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<PolledNotification>>;

    /// Look up a task's state, locking the row for the duration of the
    /// surrounding transaction. Used to safely insert a notification.
    async fn fetch_task_state_locked(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<TaskState>>;

    /// Insert a notification row.
    async fn insert_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        event: &str,
        data: Json<&RawValue>,
    ) -> sqlx::Result<PgQueryResult>;
}
