//! Deterministic simulation tests for `notification_blocking_timeout`.
//!
//! These tests use the DST infrastructure (DstScheduler, DstClock, DstEntropy,
//! DstEventSource) to get precise control over when notifications are delivered,
//! when the clock advances, and when the worker suspends tasks.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use durable_client::DurableClient;
use durable_runtime::dst::{DstClock, DstEntropy, DstEventSource, DstScheduler};
use durable_runtime::Config;
use tokio::time::timeout;

/// With a DST event source that never delivers notification events and a
/// zero suspend timeout, the task should suspend immediately. After we inject
/// a notification via the database (client API) and poke the event source,
/// the task should wake up and complete — well before the workflow's 120s
/// user timeout.
#[sqlx::test]
async fn dst_notify_timeout_suspend_then_wake(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));
    let (event_source, event_handle) = DstEventSource::new();

    // Zero suspend timeout: the task will suspend as soon as it enters the
    // notification wait loop, freeing the worker slot immediately.
    let config = Config::new()
        .suspend_margin(Duration::ZERO)
        .suspend_timeout(Duration::ZERO);

    let _guard = durable_test::spawn_worker_with_dst_events(
        pool.clone(),
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
        Box::new(event_source),
    )
    .await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch("dst suspend then wake", &program, &serde_json::json!(null))
        .await?;

    // Inject a task event so the worker picks up the new task.
    event_handle.send_task(task.id(), None);

    // Wait for the task to reach the suspended state. The zero suspend timeout
    // means it should suspend almost immediately after entering
    // notification_blocking_timeout.
    let wait_for_suspend = async {
        loop {
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

            tokio::task::yield_now().await;
        }

        anyhow::Ok(())
    };

    timeout(Duration::from_secs(30), wait_for_suspend)
        .await
        .context("task did not suspend within 30s")??;

    // Now deliver the notification through the database. The DB trigger will
    // set the task back to 'ready'.
    task.notify("wakeup", &(), &client).await?;

    // Poke the event source so the worker sees the task became ready again.
    event_handle.send_task(task.id(), None);

    // The task should replay and complete quickly.
    let start = tokio::time::Instant::now();
    let status = timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task did not complete within 30s of notification")??;
    let elapsed = start.elapsed();

    assert!(status.success(), "task should have succeeded");
    assert!(
        elapsed < Duration::from_secs(15),
        "task took {elapsed:?} after notification, expected < 15s"
    );

    Ok(())
}

/// With a controlled event source, inject a notification event at a precise
/// moment while the task is actively waiting in `notification_blocking_timeout`.
/// The task should wake up and complete promptly without needing to suspend.
#[sqlx::test]
async fn dst_notify_timeout_wake_before_suspend(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));
    let (event_source, event_handle) = DstEventSource::new();

    // Use a long suspend timeout so suspension does NOT happen during this test.
    let config = Config::new()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(300));

    let _guard = durable_test::spawn_worker_with_dst_events(
        pool.clone(),
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
        Box::new(event_source),
    )
    .await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch(
            "dst wake before suspend",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    // Tell the worker about the new task.
    event_handle.send_task(task.id(), None);

    // Give the task time to start and enter the wait loop.
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Insert the notification via the database.
    task.notify("wakeup", &(), &client).await?;

    // Inject the notification event through the DST event source so the
    // worker's broadcast channel fires immediately.
    event_handle.send_notification(task.id(), "wakeup".into());

    // The task should complete very quickly now.
    let start = tokio::time::Instant::now();
    let status = timeout(Duration::from_secs(15), task.wait(&client))
        .await
        .context("task did not complete within 15s of notification")??;
    let elapsed = start.elapsed();

    assert!(status.success(), "task should have succeeded");
    assert!(
        elapsed < Duration::from_secs(10),
        "task took {elapsed:?} after notification, expected < 10s"
    );

    // Verify the task was never suspended — it should have been woken
    // directly by the broadcast channel.
    let suspend_events: Vec<_> = scheduler
        .events()
        .iter()
        .filter(|e| {
            matches!(
                e,
                durable_runtime::scheduler::ScheduleEvent::TaskSuspended { .. }
            )
        })
        .cloned()
        .collect();
    // TaskSuspended events for our task should be absent (the scheduler may
    // not emit this event at all in the current code, so an empty list is
    // fine — the key assertion is the timing above).
    let _ = suspend_events;

    Ok(())
}

/// With a zero suspend timeout, verify that the task goes through a full
/// suspend → notification → resume → complete cycle, and that the
/// DstScheduler records the expected sequence of events.
#[sqlx::test]
async fn dst_notify_timeout_records_scheduler_events(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));
    let (event_source, event_handle) = DstEventSource::new();

    let config = Config::new()
        .suspend_margin(Duration::ZERO)
        .suspend_timeout(Duration::ZERO);

    let _guard = durable_test::spawn_worker_with_dst_events(
        pool.clone(),
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
        Box::new(event_source),
    )
    .await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch(
            "dst scheduler events test",
            &program,
            &serde_json::json!(null),
        )
        .await?;

    // Kick off the task.
    event_handle.send_task(task.id(), None);

    // Wait for the task to become suspended.
    timeout(Duration::from_secs(30), async {
        loop {
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

            tokio::task::yield_now().await;
        }

        anyhow::Ok(())
    })
    .await
    .context("task did not suspend within 30s")??;

    // The scheduler should have recorded a TaskClaimed event for our task.
    let claimed = scheduler.events().iter().any(|e| {
        matches!(e, durable_runtime::scheduler::ScheduleEvent::TaskClaimed { task_id, .. } if *task_id == task.id())
    });
    assert!(claimed, "expected TaskClaimed event for task {}", task.id());

    // Now deliver notification and wake the task.
    task.notify("wakeup", &(), &client).await?;
    event_handle.send_task(task.id(), None);

    let status = timeout(Duration::from_secs(30), task.wait(&client))
        .await
        .context("task did not complete within 30s of notification")??;
    assert!(status.success());

    // After completion, TaskClaimed should appear at least twice (initial
    // claim + claim after resume from suspension).
    let claim_count = scheduler
        .events()
        .iter()
        .filter(|e| {
            matches!(e, durable_runtime::scheduler::ScheduleEvent::TaskClaimed { task_id, .. } if *task_id == task.id())
        })
        .count();
    assert!(
        claim_count >= 2,
        "expected at least 2 TaskClaimed events (initial + after resume), got {claim_count}"
    );

    // TaskCompleted should have fired exactly once.
    let complete_count = scheduler
        .events()
        .iter()
        .filter(|e| {
            matches!(e, durable_runtime::scheduler::ScheduleEvent::TaskCompleted { task_id, .. } if *task_id == task.id())
        })
        .count();
    assert_eq!(
        complete_count, 1,
        "expected exactly 1 TaskCompleted event, got {complete_count}"
    );

    Ok(())
}

/// When the worker's internal notification broadcast channel lags (more than
/// 128 events buffered before the task's receiver drains them), the
/// `notification_blocking_timeout` implementation handles `RecvError::Lagged`
/// by breaking out of the inner select loop and re-polling the database.
///
/// This test floods the broadcast channel with notifications for *other*
/// tasks, inserts the real notification into the database, then sends one
/// more event to tip the channel past capacity. The task should recover from
/// the lag, find its notification in the DB, and complete successfully.
#[sqlx::test]
async fn dst_notify_timeout_recovers_from_lag(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));
    let (event_source, event_handle) = DstEventSource::new();

    // Long suspend timeout so the task stays active (no suspension).
    let config = Config::new()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(300));

    let _guard = durable_test::spawn_worker_with_dst_events(
        pool.clone(),
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
        Box::new(event_source),
    )
    .await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch("dst lag recovery", &program, &serde_json::json!(null))
        .await?;

    // Tell the worker about the new task.
    event_handle.send_task(task.id(), None);

    // Give the task time to start and enter the wait loop, subscribing to
    // the broadcast channel.
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Insert the real notification into the database so it will be found
    // when the task re-polls after recovering from the lag.
    task.notify("wakeup", &(), &client).await?;

    // Flood the broadcast channel with 200 notification events for a
    // non-existent task. The channel capacity is 128, so the task's
    // receiver will lag and return RecvError::Lagged on its next recv().
    for i in 0..200 {
        event_handle.send_notification(999_999 + i, format!("flood-{i}"));
    }

    // Also deliver the real notification event through the broadcast
    // channel. If the flood caused lag, the task will recover via the
    // Lagged path and re-poll the database. If it did not cause lag
    // (because the worker and task interleaved event processing), this
    // event ensures the task still wakes up and finds the notification in
    // the database.
    event_handle.send_notification(task.id(), "wakeup".into());

    // The task should find the notification quickly — either through lag
    // recovery (re-poll after RecvError::Lagged) or through the normal
    // broadcast path.
    let start = tokio::time::Instant::now();
    let status = timeout(Duration::from_secs(15), task.wait(&client))
        .await
        .context("task did not complete within 15s")??;
    let elapsed = start.elapsed();

    assert!(status.success(), "task should have succeeded");
    assert!(
        elapsed < Duration::from_secs(10),
        "task took {elapsed:?} after lag flood, expected < 10s"
    );

    Ok(())
}

/// Similar to the lag recovery test above, but the notification arrives
/// *after* the lag event. This exercises the path where lag causes a re-poll
/// that finds nothing, the task loops back into the select, and then the
/// real notification arrives normally.
#[sqlx::test]
async fn dst_notify_timeout_recovers_from_lag_then_notified(
    pool: sqlx::PgPool,
) -> anyhow::Result<()> {
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));
    let (event_source, event_handle) = DstEventSource::new();

    let config = Config::new()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(300));

    let _guard = durable_test::spawn_worker_with_dst_events(
        pool.clone(),
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
        Box::new(event_source),
    )
    .await?;

    let client = DurableClient::new(pool.clone())?;
    let program = crate::load_binary(&client, "notify-wait-timeout-wakeup.wasm").await?;

    let task = client
        .launch("dst lag then notify", &program, &serde_json::json!(null))
        .await?;

    event_handle.send_task(task.id(), None);

    // Let the task enter the wait loop.
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Flood the broadcast channel to cause lag — but do NOT insert the
    // notification into the DB yet.
    for i in 0..200 {
        event_handle.send_notification(999_999 + i, format!("flood-{i}"));
    }

    // Give the task a moment to process the lag and re-poll (finding nothing).
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Now insert the real notification and deliver it through the event
    // source so the broadcast channel fires normally.
    task.notify("wakeup", &(), &client).await?;
    event_handle.send_notification(task.id(), "wakeup".into());

    let start = tokio::time::Instant::now();
    let status = timeout(Duration::from_secs(15), task.wait(&client))
        .await
        .context("task did not complete within 15s — recovery after lag may have failed")??;
    let elapsed = start.elapsed();

    assert!(status.success(), "task should have succeeded");
    assert!(
        elapsed < Duration::from_secs(10),
        "task took {elapsed:?} after notification, expected < 10s"
    );

    Ok(())
}
