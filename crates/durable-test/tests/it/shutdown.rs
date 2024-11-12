use std::time::Duration;

#[sqlx::test]
async fn shutdown_timeout(pool: sqlx::PgPool) -> anyhow::Result<()> {
    let guard = durable_test::spawn_worker(pool.clone()).await?;

    // Give the runtime a chance to start up
    tokio::time::sleep(Duration::from_secs(1)).await;

    guard.handle().shutdown();
    match tokio::time::timeout(Duration::from_secs(5), guard).await {
        Ok(result) => result,
        Err(_) => {
            panic!("unable to shut down runtime in under 5s")
        }
    }
}
