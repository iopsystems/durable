use std::time::Duration;

use durable_client::DurableClient;

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
