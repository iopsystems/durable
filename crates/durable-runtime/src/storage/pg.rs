//! Postgres implementation of the [`Storage`](super::Storage) trait.
//!
//! Each method body is a verbatim move of the SQL that previously lived
//! inline at the call site. No semantic changes — see `storage::mod` docs.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::value::RawValue;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::PgQueryResult;
use sqlx::types::Json;
use sqlx::PgConnection;

use super::{PolledNotification, Storage, StoredEvent, TaskState, WorkerRecord};
use crate::task::RecordedEvent;
use crate::worker::TaskData;

#[derive(Clone)]
pub(crate) struct PgStorage {
    #[allow(dead_code)]
    pool: sqlx::PgPool,
}

impl PgStorage {
    pub(crate) fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Storage for PgStorage {
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    async fn insert_worker(
        &self,
        conn: &mut PgConnection,
        now: DateTime<Utc>,
    ) -> sqlx::Result<i64> {
        let record = sqlx::query!(
            "
            INSERT INTO durable.worker(heartbeat_at)
            VALUES ($1)
            RETURNING id
            ",
            now
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(record.id)
    }

    async fn delete_worker(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!("DELETE FROM durable.worker WHERE id = $1", worker_id)
            .execute(&mut *conn)
            .await
    }

    async fn heartbeat_worker(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        now: DateTime<Utc>,
    ) -> sqlx::Result<bool> {
        let record = sqlx::query!(
            "UPDATE durable.worker
              SET heartbeat_at = $2
            WHERE id = $1
            RETURNING id",
            worker_id,
            now
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.is_some())
    }

    async fn delete_following_expired_worker(
        &self,
        conn: &mut PgConnection,
        following: i64,
        timeout: PgInterval,
        now: DateTime<Utc>,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            DELETE FROM durable.worker
            WHERE id = $1
              AND $3::timestamptz - heartbeat_at > $2
            ",
            following,
            timeout,
            now
        )
        .execute(&mut *conn)
        .await
    }

    async fn delete_other_expired_workers(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        timeout: PgInterval,
        now: DateTime<Utc>,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            DELETE FROM durable.worker
            WHERE $3::timestamptz - heartbeat_at > $2
            AND NOT id = $1
            ",
            worker_id,
            timeout,
            now
        )
        .execute(&mut *conn)
        .await
    }

    async fn next_worker_in_sequence(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<Option<WorkerRecord>> {
        let record = sqlx::query!(
            r#"
            WITH
                prev AS (
                    SELECT id, heartbeat_at
                    FROM durable.worker
                    WHERE id < $1
                    ORDER BY id DESC
                    LIMIT 1
                ),
                next AS (
                    SELECT id, heartbeat_at
                    FROM durable.worker
                    WHERE NOT id = $1
                    ORDER BY id DESC
                    LIMIT 1
                ),
                combined AS (
                    SELECT * FROM prev
                    UNION ALL
                    SELECT * FROM next
                )
            SELECT
                id as "id!",
                heartbeat_at as "heartbeat_at!"
            FROM combined
            ORDER BY id ASC
            LIMIT 1
            "#,
            worker_id
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.map(|r| WorkerRecord {
            id: r.id,
            heartbeat_at: r.heartbeat_at,
        }))
    }

    async fn load_leader_id(&self, conn: &mut PgConnection) -> sqlx::Result<Option<i64>> {
        let record = sqlx::query!(
            "
            SELECT id
             FROM durable.worker
            ORDER BY started_at ASC, id ASC
            LIMIT 1
            "
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.map(|r| r.id))
    }

    async fn wake_suspended_tasks(
        &self,
        conn: &mut PgConnection,
        suspend_margin: PgInterval,
        now: DateTime<Utc>,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            UPDATE durable.task
              SET state = 'ready',
                  wakeup_at = NULL,
                  running_on = (
                    SELECT id
                     FROM durable.worker
                    ORDER BY random() + task.id
                    LIMIT 1
                  )
            WHERE state = 'suspended'
              AND wakeup_at <= ($2::timestamptz - $1::interval)
            ",
            suspend_margin,
            now
        )
        .execute(&mut *conn)
        .await
    }

    async fn next_wakeup_at(&self, conn: &mut PgConnection) -> sqlx::Result<Option<DateTime<Utc>>> {
        let record = sqlx::query!(
            r#"
            SELECT wakeup_at as "wakeup_at!"
             FROM durable.task
            WHERE state = 'suspended'
              AND wakeup_at IS NOT NULL
            ORDER BY wakeup_at ASC
            LIMIT 1
            "#
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.map(|r| r.wakeup_at))
    }

    async fn cleanup_old_tasks(
        &self,
        conn: &mut PgConnection,
        cleanup_age: PgInterval,
        limit: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            DELETE FROM durable.task
            WHERE task.ctid = ANY(ARRAY(
                SELECT ctid
                FROM durable.task
                WHERE completed_at < NOW() - $1::interval
                LIMIT $2
                FOR UPDATE
            ))
            "#,
            cleanup_age,
            limit
        )
        .execute(&mut *conn)
        .await
    }

    async fn unwedge_stuck_notifications(
        &self,
        conn: &mut PgConnection,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            UPDATE durable.task
              SET state = 'ready'
            WHERE state = 'suspended'
              AND EXISTS((
                SELECT task_id
                 FROM durable.notification
                WHERE task_id = task.id
                  AND created_at < NOW() - '10 minutes'::interval
              ))
            "#
        )
        .execute(&mut *conn)
        .await
    }

    async fn reset_failed_tasks(
        &self,
        conn: &mut PgConnection,
        failed_ids: &[i64],
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            UPDATE durable.task
              SET state = 'ready',
                  running_on = NULL
            WHERE id = ANY($1::bigint[])
              AND running_on = $2
            ",
            failed_ids,
            worker_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn claim_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
        allowed: i64,
    ) -> sqlx::Result<Vec<TaskData>> {
        sqlx::query_as!(
            TaskData,
            r#"
            WITH selected AS (
                SELECT id
                 FROM durable.task
                WHERE (state IN ('ready', 'active') AND running_on IS NULL)
                   OR (state = 'ready' AND running_on = $1)
                ORDER BY id ASC
                FOR NO KEY UPDATE SKIP LOCKED
                LIMIT $2
            )
            UPDATE durable.task
              SET running_on = $1,
                  state = 'active'
             FROM selected
            WHERE selected.id = task.id
            RETURNING
                task.id         as id,
                task.name       as name,
                task.created_at as created_at,
                task.wasm       as "wasm!",
                task.data       as "data!: Json<Box<RawValue>>"
            "#,
            worker_id,
            allowed
        )
        .fetch_all(&mut *conn)
        .await
    }

    async fn release_owned_ready_tasks(
        &self,
        conn: &mut PgConnection,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            UPDATE durable.task
              SET running_on = NULL
            WHERE state = 'ready'
              AND running_on = $1
            ",
            worker_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn reset_task_for_retry(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        worker_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "
            UPDATE durable.task
            SET state = 'ready',
                running_on = NULL
            WHERE id = $1
              AND running_on = $2
            ",
            task_id,
            worker_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn mark_task_complete(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "UPDATE durable.task
            SET state = 'complete',
                completed_at = CURRENT_TIMESTAMP,
                running_on = NULL,
                wasm = NULL
            WHERE id = $1
            ",
            task_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn mark_task_failed(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "UPDATE durable.task
            SET state = 'failed',
                completed_at = CURRENT_TIMESTAMP,
                running_on = NULL,
                wasm = NULL
            WHERE id = $1",
            task_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn fetch_wasm_blob(
        &self,
        conn: &mut PgConnection,
        wasm_id: i64,
    ) -> sqlx::Result<Vec<u8>> {
        let record = sqlx::query!("SELECT wasm FROM durable.wasm WHERE id = $1", wasm_id)
            .fetch_one(&mut *conn)
            .await?;

        Ok(record.wasm)
    }

    async fn fetch_recorded_events(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Vec<RecordedEvent>> {
        sqlx::query_as!(
            RecordedEvent,
            r#"
            SELECT
                index,
                label,
                value as "value!: Json<Box<RawValue>>"
             FROM durable.event
            WHERE task_id = $1
            ORDER BY index ASC
            LIMIT 1000
            "#,
            task_id
        )
        .fetch_all(&mut *conn)
        .await
    }

    async fn fetch_event_at_index(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
    ) -> sqlx::Result<Option<StoredEvent>> {
        let record = sqlx::query!(
            r#"
            SELECT
                label,
                value as "value: Json<Box<RawValue>>"
             FROM durable.event
            WHERE task_id = $1
              AND index = $2
            "#,
            task_id,
            index
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.map(|r| StoredEvent {
            label: r.label,
            value: r.value,
        }))
    }

    async fn commit_event_with_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        label: &str,
        value: Json<&RawValue>,
        message: Option<&str>,
    ) -> sqlx::Result<Option<i64>> {
        let record = sqlx::query!(
            r#"
            WITH
                current_task AS (
                    SELECT id, running_on
                    FROM durable.task
                    WHERE id = $1
                    LIMIT 1
                ),
                insert_event AS (
                    INSERT INTO durable.event(task_id, index, label, value)
                    SELECT
                        id as task_id,
                        $2 as index,
                        $3 as label,
                        $4 as value
                    FROM current_task
                    RETURNING task_id
                ),
                insert_log AS (
                    INSERT INTO durable.log(task_id, index, message)
                    SELECT task_id, index, message
                    FROM (VALUES ($1, $2, $5)) as t(task_id, index, message)
                    JOIN current_task task ON task.id = task_id
                    WHERE message IS NOT NULL
                    RETURNING task_id
                )
            SELECT running_on
             FROM current_task
            LEFT JOIN insert_event event ON event.task_id = id
            LEFT JOIN insert_event log   ON log.task_id = id
            "#,
            task_id,
            index,
            label,
            value as Json<&RawValue>,
            message
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(record.running_on)
    }

    async fn suspend_task(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        wakeup_at: Option<DateTime<Utc>>,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "UPDATE durable.task
            SET state = 'suspended',
                running_on = NULL,
                wakeup_at = $2
            WHERE id = $1",
            task_id,
            wakeup_at
        )
        .execute(&mut *conn)
        .await
    }

    async fn suspend_task_no_wakeup(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "UPDATE durable.task
              SET state = 'suspended',
                  running_on = NULL
            WHERE id = $1
            ",
            task_id
        )
        .execute(&mut *conn)
        .await
    }

    async fn insert_log(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO durable.log(task_id, index, message)
             VALUES ($1, $2, $3)",
            task_id,
            index,
            message
        )
        .execute(&mut *conn)
        .await
    }

    async fn upsert_log_error(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        index: i32,
        message: &str,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            "INSERT INTO durable.log(task_id, index, message)
             VALUES ($1, $2, $3)
             ON CONFLICT ON CONSTRAINT log_pkey DO UPDATE
             SET message = $3
             ",
            task_id,
            index,
            message
        )
        .execute(&mut *conn)
        .await
    }

    async fn poll_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<PolledNotification>> {
        let record = sqlx::query!(
            r#"
            DELETE FROM durable.notification
            WHERE ctid IN (
                SELECT ctid
                 FROM durable.notification
                WHERE task_id = $1
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE
            )
            RETURNING
                created_at,
                event,
                data as "data: Json<Box<RawValue>>"
            "#,
            task_id
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(record.map(|r| PolledNotification {
            created_at: r.created_at,
            event: r.event,
            data: r.data,
        }))
    }

    async fn fetch_task_state_locked(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
    ) -> sqlx::Result<Option<TaskState>> {
        sqlx::query_scalar!(
            r#"
            SELECT state as "state!: TaskState"
             FROM durable.task
            WHERE task.id = $1
            FOR UPDATE
            "#,
            task_id
        )
        .fetch_optional(&mut *conn)
        .await
    }

    async fn insert_notification(
        &self,
        conn: &mut PgConnection,
        task_id: i64,
        event: &str,
        data: Json<&RawValue>,
    ) -> sqlx::Result<PgQueryResult> {
        sqlx::query!(
            r#"
            INSERT INTO durable.notification(task_id, event, data)
            VALUES ($1, $2, $3)
            "#,
            task_id,
            event,
            data as Json<&RawValue>
        )
        .execute(&mut *conn)
        .await
    }
}
