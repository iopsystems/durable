use std::time::Duration;

use anyhow::Context;
use durable_client::DurableClient;
use durable_runtime::Config;
use sqlx::postgres::PgListener;
use tokio::time::timeout;

#[sqlx::test]
async fn notify_self(pool: sqlx::PgPool) -> anyhow::Result<()> {
    use tokio::time::timeout;

    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "notify-self.wasm").await?;

    let task = client
        .launch("notify self test", &program, &serde_json::json!(null))
        .await?;

    let status = match timeout(Duration::from_secs(30), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!("task failed to complete in under 30s"),
    };
    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn notify_in_advance(pool: sqlx::PgPool) -> anyhow::Result<()> {
    use tokio::time::timeout;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait.wasm").await?;
    let task = client
        .launch("notify self test", &program, &serde_json::json!(null))
        .await?;

    task.notify("notification", &(), &client).await?;

    let _guard = durable_test::spawn_worker(pool.clone()).await?;

    let status = match timeout(Duration::from_secs(30), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!("task failed to complete in under 30s"),
    };
    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn notify_after_suspend(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait.wasm").await?;
    let task = client
        .launch("notify self test", &program, &serde_json::json!(null))
        .await?;

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("durable:task-suspend").await?;

    let _guard = durable_test::spawn_worker_with(
        pool.clone(),
        Config::new()
            .suspend_margin(Duration::ZERO)
            .suspend_timeout(Duration::ZERO),
    )
    .await?;

    let future = async {
        loop {
            let _ = listener.try_recv().await?;

            let suspended = sqlx::query_scalar!(
                r#"
                SELECT state = 'suspended' as "state!"
                FROM durable.task
                WHERE id = $1
                "#,
                task.id()
            )
            .fetch_one(&pool)
            .await?;

            if suspended {
                break;
            }
        }

        anyhow::Ok(())
    };

    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .context("task failed to suspend itself within 30s")??;

    task.notify("notification", &(), &client).await?;

    let status = tokio::time::timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task failed to complete in under 30s")??;
    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn notify_multiple_workers(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait.wasm").await?;
    let task = client
        .launch("notify self test", &program, &serde_json::json!(null))
        .await?;

    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("durable:task-suspend").await?;

    let _guard1 = durable_test::spawn_worker_with(
        pool.clone(),
        Config::new()
            .suspend_margin(Duration::ZERO)
            .suspend_timeout(Duration::ZERO),
    )
    .await?;

    let _guard2 = durable_test::spawn_worker_with(
        pool.clone(),
        Config::new()
            .suspend_margin(Duration::ZERO)
            .suspend_timeout(Duration::ZERO),
    )
    .await?;

    let future = async {
        loop {
            let _ = listener.try_recv().await?;

            let suspended = sqlx::query_scalar!(
                r#"
                SELECT state = 'suspended' as "state!"
                FROM durable.task
                WHERE id = $1
                "#,
                task.id()
            )
            .fetch_one(&pool)
            .await?;

            if suspended {
                break;
            }
        }

        anyhow::Ok(())
    };

    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .context("task failed to suspend itself within 30s")??;

    task.notify("notification", &(), &client).await?;

    let status = tokio::time::timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task failed to complete in under 30s")??;
    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn notify_wait_timeout(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "notify-wait-timeout.wasm").await?;

    let task = client
        .launch(
            "notify wait timeout test",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    let status = match timeout(Duration::from_secs(60), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!("task failed to complete in under 60s"),
    };
    assert!(status.success());

    Ok(())
}

/// Verify that `wait_with_timeout` wakes up promptly when a notification
/// arrives mid-wait, rather than sleeping until the timeout expires.
#[sqlx::test]
async fn notify_wait_timeout_wakeup(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch(
            "notify wait timeout wakeup test",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    // Give the task a moment to start waiting, then deliver the notification.
    tokio::time::sleep(Duration::from_secs(2)).await;
    task.notify("wakeup", &(), &client).await?;

    // The workflow has a 120s timeout. If wake-up works, it should complete
    // well within 15s of us sending the notification.
    let start = tokio::time::Instant::now();
    let status = match timeout(Duration::from_secs(15), task.wait(&client)).await {
        Ok(result) => result?,
        Err(_) => anyhow::bail!(
            "task did not complete within 15s of notification — wake-up is not timely"
        ),
    };
    let elapsed = start.elapsed();
    assert!(status.success());

    // Sanity check: it should have completed in a few seconds, not anywhere
    // near the 120s timeout the workflow specified.
    assert!(
        elapsed < Duration::from_secs(10),
        "task took {elapsed:?} to complete after notification, expected < 10s"
    );

    Ok(())
}

/// Verify that `wait_with_timeout` correctly suspends the task when the
/// suspend timeout fires before the user timeout, and that sending a
/// notification after suspension wakes the task up and delivers the result
/// in a timely manner.
#[sqlx::test]
async fn notify_wait_timeout_after_suspend(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch(
            "notify wait timeout after suspend test",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    // Listen for the task-suspend event so we know when it's actually
    // suspended.
    let mut listener = PgListener::connect_with(&pool).await?;
    listener.listen("durable:task-suspend").await?;

    // Use a zero suspend timeout to force the task to suspend immediately
    // rather than holding a worker slot.
    let _guard = durable_test::spawn_worker_with(
        pool.clone(),
        Config::new()
            .suspend_margin(Duration::ZERO)
            .suspend_timeout(Duration::ZERO),
    )
    .await?;

    // Wait until the task is confirmed suspended.
    let wait_for_suspend = async {
        loop {
            let _ = listener.try_recv().await?;

            let suspended = sqlx::query_scalar!(
                r#"
                SELECT state = 'suspended' as "state!"
                FROM durable.task
                WHERE id = $1
                "#,
                task.id()
            )
            .fetch_one(&pool)
            .await?;

            if suspended {
                break;
            }
        }

        anyhow::Ok(())
    };

    tokio::time::timeout(Duration::from_secs(30), wait_for_suspend)
        .await
        .context("task failed to suspend itself within 30s")??;

    // Now send a notification. The DB trigger should wake the task from
    // suspension and a worker should pick it up.
    task.notify("wakeup", &(), &client).await?;

    // The workflow specified a 120s user timeout. If the suspend/wake path
    // works, the task should complete well within 30s of the notification.
    let start = tokio::time::Instant::now();
    let status = tokio::time::timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task failed to complete within 30s of notification")?;
    let status = status?;
    let elapsed = start.elapsed();

    assert!(status.success());
    assert!(
        elapsed < Duration::from_secs(15),
        "task took {elapsed:?} to complete after notification, expected < 15s"
    );

    Ok(())
}

/// A notification inserted by a transaction that is still open when the
/// worker tries to suspend the task must still wake the task.
///
/// The insert's `notify_notification` trigger runs while the task is active,
/// so it does not wake anything, and the row is invisible to the worker's
/// pre-suspend poll until the transaction commits. The suspend has to wait for
/// that commit; otherwise the task is suspended with a notification pending
/// and nothing wakes it until the stuck-notification sweep.
#[sqlx::test]
async fn notify_uncommitted_during_suspend(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait.wasm").await?;
    let task = client
        .launch(
            "notify uncommitted test",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    // Insert the notification but leave the transaction open.
    let mut tx = pool.begin().await?;
    task.notify_with("notification", &(), &mut *tx).await?;

    // A zero suspend timeout sends the worker straight to the suspend path.
    let _guard = durable_test::spawn_worker_with(
        pool.clone(),
        Config::new()
            .suspend_margin(Duration::ZERO)
            .suspend_timeout(Duration::ZERO),
    )
    .await?;

    // Wait until the worker has either suspended the task, which is the race
    // being tested for, or is blocked on the task row lock held by the open
    // insert, which is the fix. Committing before either would let the
    // worker's ordinary poll find the notification and test nothing.
    let reached_suspend = async {
        loop {
            let (suspended, blocked): (bool, bool) = sqlx::query_as(
                r#"
                SELECT
                    (SELECT state = 'suspended' FROM durable.task WHERE id = $1),
                    EXISTS(
                        SELECT 1
                         FROM pg_stat_activity
                        WHERE datname = current_database()
                          AND wait_event_type = 'Lock'
                          AND query LIKE '%FROM durable.task%FOR UPDATE%'
                    )
                "#,
            )
            .bind(task.id())
            .fetch_one(&pool)
            .await?;

            if suspended || blocked {
                break;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        anyhow::Ok(())
    };

    timeout(Duration::from_secs(30), reached_suspend)
        .await
        .context("worker never reached the suspend path within 30s")??;

    tx.commit().await?;

    let status = timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task was not woken by a notification committed during its suspend")??;
    assert!(status.success());

    Ok(())
}
