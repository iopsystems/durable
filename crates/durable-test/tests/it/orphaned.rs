//! Tests for tasks orphaned by a worker that died without cleaning up after
//! itself.

use std::time::Duration;

use durable_client::DurableClient;
use tokio::time::timeout;

/// Register a worker row that no worker is actually behind, and return its id.
///
/// This stands in for a worker that was killed outright: `Worker::run` deletes
/// its own row on the way out, so a row with nothing behind it is exactly what
/// a SIGKILL or an OOM kill leaves in the database.
async fn insert_dead_worker(pool: &sqlx::PgPool) -> anyhow::Result<i64> {
    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO durable.worker(heartbeat_at) VALUES (CURRENT_TIMESTAMP) RETURNING id",
    )
    .fetch_one(pool)
    .await?;

    Ok(id)
}

async fn task_state(pool: &sqlx::PgPool, task_id: i64) -> anyhow::Result<String> {
    let state =
        sqlx::query_scalar::<_, String>("SELECT state::text FROM durable.task WHERE id = $1")
            .bind(task_id)
            .fetch_one(pool)
            .await?;

    Ok(state)
}

/// A task whose worker died mid-execution must be picked up by a live worker
/// once the dead worker is reaped.
///
/// The reap is what hands the task back: `fk_worker` is `ON DELETE SET NULL`,
/// so removing the worker row clears `running_on` and leaves the task claimable
/// at `state = 'active'`. Nothing polls for such tasks — a worker only looks
/// for work when an event arrives — so the release has to announce itself, or
/// the task sits there until a worker happens to start up.
#[sqlx::test]
async fn orphaned_task_runs_after_dead_worker_is_reaped(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "task-details.wasm").await?;

    let task = client
        .launch("orphaned task", &program, &serde_json::json!(null))
        .await?;

    // Pin the task to a worker that is not coming back, as it would have been
    // left had that worker been killed while running it.
    let dead_worker = insert_dead_worker(&pool).await?;
    sqlx::query("UPDATE durable.task SET state = 'active', running_on = $1 WHERE id = $2")
        .bind(dead_worker)
        .bind(task.id())
        .execute(&pool)
        .await?;

    let _guard = durable_test::spawn_worker(pool.clone()).await?;

    // The live worker must leave the task alone while it still belongs to
    // someone else, otherwise two workers could run it at once.
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert_eq!(
        task_state(&pool, task.id()).await?,
        "active",
        "a task owned by another worker was claimed while that worker still held it"
    );

    // Reap the dead worker, which is what `validate_workers` does once its
    // heartbeat expires.
    sqlx::query("DELETE FROM durable.worker WHERE id = $1")
        .bind(dead_worker)
        .execute(&pool)
        .await?;

    let status = match timeout(Duration::from_secs(30), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!(
            "the orphaned task was never claimed after its dead worker was reaped; releasing \
             `running_on` did not notify any live worker"
        ),
    };
    assert!(status.success());

    Ok(())
}

/// The same recovery, but for a task that was only `ready` on the dead worker
/// rather than `active`.
///
/// `claim_tasks` accepts both states when `running_on` is NULL, so both have to
/// be announced when they are released.
#[sqlx::test]
async fn orphaned_ready_task_runs_after_dead_worker_is_reaped(
    pool: sqlx::PgPool,
) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "task-details.wasm").await?;

    let task = client
        .launch("orphaned ready task", &program, &serde_json::json!(null))
        .await?;

    let dead_worker = insert_dead_worker(&pool).await?;
    sqlx::query("UPDATE durable.task SET state = 'ready', running_on = $1 WHERE id = $2")
        .bind(dead_worker)
        .bind(task.id())
        .execute(&pool)
        .await?;

    let _guard = durable_test::spawn_worker(pool.clone()).await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    sqlx::query("DELETE FROM durable.worker WHERE id = $1")
        .bind(dead_worker)
        .execute(&pool)
        .await?;

    let status = match timeout(Duration::from_secs(30), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!(
            "the orphaned task was never claimed after its dead worker was reaped; releasing \
             `running_on` did not notify any live worker"
        ),
    };
    assert!(status.success());

    Ok(())
}
