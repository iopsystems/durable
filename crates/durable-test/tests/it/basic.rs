use std::time::Duration;

use durable_client::DurableClient;
use futures::TryStreamExt;

#[sqlx::test]
async fn check_task_details(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "task-details.wasm").await?;

    let task = client
        .launch("test task", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    let logs = task
        .read_logs(&client)
        .try_fold(String::new(), |mut acc, item| {
            acc.push_str(&item);
            std::future::ready(Ok(acc))
        })
        .await?;

    assert_eq!(
        logs,
        format!(
            "\
Task Details:
    id:   {}
    name: test task
    data: null
",
            task.id(),
        )
    );

    Ok(())
}

#[sqlx::test(fixtures("extra-table"))]
async fn run_sqlx_test(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-use-test-data.wasm").await?;

    let task = client
        .launch("test task", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn run_sqlx_enum(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-enum.wasm").await?;

    let task = client
        .launch("sqlx enum test", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn run_sqlx_inet(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-inet.wasm").await?;

    let task = client
        .launch("sqlx inet test", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn run_sqlx_macros_test(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-test-macros.wasm").await?;

    let task = client
        .launch("wasm macros test", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn run_sqlx_use_json(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-use-json.wasm").await?;

    let task = client
        .launch("sqlx json types test", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}

#[sqlx::test]
async fn run_notify_self(pool: sqlx::PgPool) -> anyhow::Result<()> {
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
