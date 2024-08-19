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
    crate::tail_logs(&client, &task);
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
    crate::tail_logs(&client, &task);
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}
