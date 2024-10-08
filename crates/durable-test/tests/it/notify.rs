use std::time::Duration;

use anyhow::Context;
use durable_client::DurableClient;
use durable_runtime::Config;
use sqlx::postgres::PgListener;

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
