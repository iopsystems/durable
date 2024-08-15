use anyhow::Context;
use durable_client::{DurableClient, ProgramOptions};
use durable_runtime::WorkerBuilder;
use futures::TryStreamExt;

#[sqlx::test]
async fn check_task_details(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let mut worker = WorkerBuilder::new(pool.clone())
        .config(crate::test_config())
        .validate_database(false)
        .build()
        .await?;
    let handle = worker.handle();

    tokio::spawn(async move {
        let _ = worker.run().await;
    });

    let client = DurableClient::new(pool)?;
    let program = client
        .program(
            ProgramOptions::from_file(crate::test_binary("task-details.wasm"))
                .context("failed to load task-details.wasm file")?,
        )
        .await?;

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

    handle.shutdown();

    Ok(())
}
