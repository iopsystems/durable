//! Tests for task executions that keep running on a worker after another
//! worker has taken their task.
//!
//! This happens when a worker misses its heartbeats for `heartbeat_timeout`
//! but keeps running. Another worker deletes its row, which clears
//! `running_on`, and claims the task. The old worker's executions keep going
//! until its next heartbeat notices that its row is gone. Every write those
//! executions make to their task must check that they still own it.
//!
//! The tests below hand the task over by pointing `running_on` at a worker
//! row that has no worker behind it, which is what the task looks like from
//! the database once another worker has claimed it.

use std::time::Duration;

use anyhow::Context;
use durable_client::{DurableClient, Task};
use durable_runtime::Config;
use futures::TryStreamExt;
use tokio::time::timeout;

/// Register a worker row that no worker is actually behind, and return its id.
///
/// This stands in for the worker that took over the task.
async fn insert_other_worker(pool: &sqlx::PgPool) -> anyhow::Result<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO durable.worker(heartbeat_at) VALUES (CURRENT_TIMESTAMP) RETURNING id",
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

async fn task_owner(pool: &sqlx::PgPool, task_id: i64) -> anyhow::Result<(String, Option<i64>)> {
    let row = sqlx::query_as::<_, (String, Option<i64>)>(
        "SELECT state::text, running_on FROM durable.task WHERE id = $1",
    )
    .bind(task_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

/// Wait until the task is active on a worker and return that worker's id.
async fn wait_until_active(pool: &sqlx::PgPool, task_id: i64) -> anyhow::Result<i64> {
    let wait = async {
        loop {
            let (state, running_on) = task_owner(pool, task_id).await?;
            if let ("active", Some(worker_id)) = (state.as_str(), running_on) {
                return anyhow::Ok(worker_id);
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };

    timeout(Duration::from_secs(30), wait)
        .await
        .context("task was not claimed by the worker within 30s")?
}

/// Wait until the task has recorded at least `count` events.
async fn wait_for_events(pool: &sqlx::PgPool, task_id: i64, count: i64) -> anyhow::Result<()> {
    let wait = async {
        loop {
            let recorded = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM durable.event WHERE task_id = $1",
            )
            .bind(task_id)
            .fetch_one(pool)
            .await?;

            if recorded >= count {
                return anyhow::Ok(());
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    };

    timeout(Duration::from_secs(30), wait)
        .await
        .context("task did not record its events within 30s")?
}

/// Hand the task over to `worker_id` without changing its state, as claiming it
/// on another worker would.
async fn reassign(pool: &sqlx::PgPool, task_id: i64, worker_id: i64) -> anyhow::Result<()> {
    sqlx::query("UPDATE durable.task SET running_on = $1 WHERE id = $2")
        .bind(worker_id)
        .bind(task_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Wait for every execution the worker started before now to have finished.
///
/// The worker runs with `max_tasks = 1`, so it can only claim this new task
/// once the execution it is already running has returned.
async fn wait_for_worker_to_drain(
    client: &DurableClient,
    pool: &sqlx::PgPool,
) -> anyhow::Result<()> {
    let program = crate::load_binary(client, "task-details.wasm").await?;
    let task = client
        .launch("drain the worker", &program, &serde_json::json!(null))
        .await?;

    // Launching assigns the task to a random worker, which may be the one with
    // nothing behind it. Release it so that the real worker can claim it.
    sqlx::query("UPDATE durable.task SET running_on = NULL WHERE id = $1")
        .bind(task.id())
        .execute(pool)
        .await?;

    let status = timeout(Duration::from_secs(30), task.wait(client))
        .await
        .context("the stale execution did not finish within 30s")??;
    assert!(status.success());

    Ok(())
}

async fn wait_for_success(client: &DurableClient, task: &Task) -> anyhow::Result<()> {
    let status = timeout(Duration::from_secs(30), task.wait(client))
        .await
        .context("task did not complete within 30s")??;
    assert!(status.success(), "task did not complete successfully");

    Ok(())
}

/// A stale execution that times out waiting for a notification must not
/// suspend a task that another worker now owns (#125).
///
/// The new owner may have consumed the notification already. Suspending the
/// task then leaves it with no `wakeup_at` and no pending notification, and
/// the new owner's next commit sees `running_on = NULL` and gives up on it.
/// Nothing ever wakes the task again.
#[sqlx::test]
async fn stale_execution_does_not_suspend_reowned_task(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let config = Config::new()
        .max_tasks(1)
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(2));
    let _guard = durable_test::spawn_worker_with(pool.clone(), config).await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait.wasm").await?;
    let task = client
        .launch("stale suspend", &program, &serde_json::json!(null))
        .await?;

    wait_until_active(&pool, task.id()).await?;

    let other = insert_other_worker(&pool).await?;
    reassign(&pool, task.id(), other).await?;

    // The execution reaches suspend_timeout and calls try_suspend.
    wait_for_worker_to_drain(&client, &pool).await?;

    assert_eq!(
        task_owner(&pool, task.id()).await?,
        ("active".to_owned(), Some(other)),
        "a stale execution suspended a task owned by another worker"
    );

    // Once the other worker is reaped the task is ours again and completes.
    sqlx::query("DELETE FROM durable.worker WHERE id = $1")
        .bind(other)
        .execute(&pool)
        .await?;
    task.notify("wakeup", &(), &client).await?;

    wait_for_success(&client, &task).await
}

/// A stale execution must not record events for a task that another worker
/// now owns (#126).
///
/// If it did, the new owner's insert of the same event would conflict with it.
#[sqlx::test]
async fn stale_execution_does_not_record_events(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let config = Config::new()
        .max_tasks(1)
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(60));
    let _guard = durable_test::spawn_worker_with(pool.clone(), config).await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-short-timeout.wasm").await?;
    let task = client
        .launch("stale commit", &program, &serde_json::json!(null))
        .await?;

    wait_until_active(&pool, task.id()).await?;

    let other = insert_other_worker(&pool).await?;
    reassign(&pool, task.id(), other).await?;

    // The wait times out and the execution tries to record that, outside of a
    // database transaction.
    wait_for_worker_to_drain(&client, &pool).await?;

    let recorded =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM durable.event WHERE task_id = $1")
            .bind(task.id())
            .fetch_one(&pool)
            .await?;
    assert_eq!(
        recorded, 0,
        "a stale execution recorded an event for a task owned by another worker"
    );
    assert_eq!(
        task_owner(&pool, task.id()).await?,
        ("active".to_owned(), Some(other)),
    );

    sqlx::query("DELETE FROM durable.worker WHERE id = $1")
        .bind(other)
        .execute(&pool)
        .await?;

    wait_for_success(&client, &task).await
}

/// If a stale execution records an event before the task's owner does, the
/// owner has lost a race, not failed. It must retry the task from the recorded
/// events instead of failing it (#126).
///
/// The `running_on` check in `commit_event_with_log` uses the statement's
/// snapshot, so a stale execution can still record an event that the new
/// owner then also tries to record. The test inserts that event directly.
#[sqlx::test]
async fn owner_retries_after_losing_event_race(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let config = Config::new()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(60));
    let _guard = durable_test::spawn_worker_with(pool.clone(), config).await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "print-then-wait.wasm").await?;
    let task = client
        .launch("lost event race", &program, &serde_json::json!(null))
        .await?;

    // Event 0 is the print. Once it is recorded the task moves straight on to
    // waiting for a notification at index 1.
    wait_for_events(&pool, task.id(), 1).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;

    // The stale execution received a different notification and recorded it.
    sqlx::query(
        "INSERT INTO durable.event(task_id, index, label, value)
         VALUES ($1, 1, 'durable:core/notify.notification-blocking', $2::jsonb)",
    )
    .bind(task.id())
    .bind(
        serde_json::json!({
            "created_at": "2026-01-01T00:00:00Z",
            "event": "stale",
            "data": "from the stale execution",
        })
        .to_string(),
    )
    .execute(&pool)
    .await?;

    // The owner receives its own notification and tries to record it at the
    // same index.
    task.notify("owner", &(), &client).await?;

    wait_for_success(&client, &task).await?;

    // The retry replayed the event the stale execution recorded...
    let logs = task
        .read_logs(&client)
        .try_fold(String::new(), |mut acc, item| {
            acc.push_str(&item);
            std::future::ready(Ok(acc))
        })
        .await?;
    assert!(
        logs.contains("event: stale"),
        "the retry did not replay the recorded event; logs were:\n{logs}"
    );

    // ...and the owner's notification was rolled back along with its commit.
    let pending = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM durable.notification WHERE task_id = $1",
    )
    .bind(task.id())
    .fetch_one(&pool)
    .await?;
    assert_eq!(pending, 1);

    Ok(())
}
