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

#[derive(Copy, Clone, Debug, Eq, PartialEq, sqlx::Type)]
#[sqlx(type_name = "durable.task_state", rename_all = "lowercase")]
pub(crate) enum TaskState {
    Ready,
    Active,
    Suspended,
    Complete,
    Failed,
}

#[derive(Debug)]
pub(crate) struct WorkerRecord {
    pub id: i64,
    pub heartbeat_at: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct StoredEvent {
    pub label: String,
    pub value: Json<Box<RawValue>>,
}

#[derive(Debug)]
pub(crate) struct PolledNotification {
    pub created_at: DateTime<Utc>,
    pub event: String,
    pub data: Json<Box<RawValue>>,
}

#[async_trait]
pub(crate) trait Storage: Send + Sync + 'static {
    /// Currently unused — transaction lifecycle is driven through
    /// `SharedState::pool` until phase 2 of the storage abstraction.
    #[allow(dead_code)]
    fn pool(&self) -> &sqlx::PgPool;

    async fn insert_worker(&self, conn: &mut PgConnection) -> sqlx::Result<i64>;

    async fn delete_worker(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Returns `true` if the row still exists.
    async fn heartbeat_worker(&self, conn: &mut PgConnection, worker_id: i64)
        -> sqlx::Result<bool>;

    async fn delete_following_expired_worker(
        &self,
        conn: &mut PgConnection,
        following: i64,
        timeout: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    async fn delete_other_expired_workers(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        timeout: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    /// See `Worker::validate_workers` for the algorithm.
    async fn next_worker_in_sequence(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<Option<WorkerRecord>>;

    /// The leader is the oldest worker.
    async fn load_leader_id(&self, conn: &mut PgConnection) -> sqlx::Result<Option<i64>>;

    async fn wake_suspended_tasks(
        &self,
        conn: &mut PgConnection,
        suspend_margin: PgInterval,
    ) -> sqlx::Result<PgQueryResult>;

    async fn next_wakeup_at(&self, conn: &mut PgConnection) -> sqlx::Result<Option<DateTime<Utc>>>;

    async fn cleanup_old_tasks(
        &self,
        conn: &mut PgConnection,
        cleanup_age: PgInterval,
        limit: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn unwedge_stuck_notifications(
        &self,
        conn: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult>;

    async fn reset_failed_tasks(
        &self,
        conn: &mut PgConnection,
        failed_ids: &[i64],
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn claim_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        allowed: i64,
    ) -> sqlx::Result<Vec<TaskData>>;

    /// Used when a worker hits its capacity before committing a claim.
    async fn release_owned_ready_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    /// Only resets the task if it is still owned by `worker_id`.
    async fn reset_task_for_retry(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn mark_task_complete(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn mark_task_failed(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn fetch_wasm_blob(&self, conn: &mut PgConnection, wasm_id: i64)
        -> sqlx::Result<Vec<u8>>;

    async fn fetch_recorded_events(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Vec<RecordedEvent>>;

    async fn fetch_event_at_index(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
    ) -> sqlx::Result<Option<StoredEvent>>;

    /// Inserts an event row, optionally inserts a log row, and returns the
    /// task's `running_on` value in a single round-trip.
    async fn commit_event_with_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        label: &str,
        value: Json<&RawValue>,
        message: Option<&str>,
    ) -> sqlx::Result<Option<i64>>;

    async fn suspend_task(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        wakeup_at: Option<DateTime<Utc>>,
    ) -> sqlx::Result<PgQueryResult>;

    /// Suspends without touching `wakeup_at`. Used by the notification
    /// blocking path, which must not clobber an existing wakeup deadline.
    async fn suspend_task_no_wakeup(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult>;

    async fn insert_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult>;

    /// Replaces the message if a row already exists at `(task_id, index)`.
    /// Used to record the final error message for a task.
    async fn upsert_log_error(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult>;

    async fn poll_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<PolledNotification>>;

    /// Locks the row for the duration of the surrounding transaction so a
    /// concurrent notification poll cannot consume the task.
    async fn fetch_task_state_locked(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<TaskState>>;

    async fn insert_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        event: &str,
        data: Json<&RawValue>,
    ) -> sqlx::Result<PgQueryResult>;
}
