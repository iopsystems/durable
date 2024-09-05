use durable_client::DurableClient;

#[sqlx::test(fixtures("extra-table"))]
async fn enum_insert(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let _guard = durable_test::spawn_worker(pool.clone()).await?;
    let client = DurableClient::new(pool)?;
    let program = crate::load_binary(&client, "sqlx-enum-insert.wasm").await?;

    let task = client
        .launch("enum insert test", &program, &serde_json::json!(null))
        .await?;
    let status = task.wait(&client).await?;

    assert!(status.success());

    Ok(())
}
