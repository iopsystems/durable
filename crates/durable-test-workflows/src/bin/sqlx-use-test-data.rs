use durable::sqlx;

fn main() -> anyhow::Result<()> {
    let task = durable::task();

    let result = sqlx::transaction("insert a thing", |mut conn| {
        sqlx::query("INSERT INTO test_data(label, value) VALUES ($1, $2)")
            .bind(task.name())
            .bind(task.id().to_string())
            .execute(&mut conn)
    })?;

    assert_eq!(result.rows_affected(), 1);

    let row = sqlx::transaction("now read it back", |mut conn| {
        sqlx::query("SELECT label FROM test_data").fetch_one(&mut conn)
    })?;

    let name1: String = row.get("label");
    let name2: String = row.get(0);

    assert_eq!(task.name(), name1);
    assert_eq!(task.name(), name2);

    Ok(())
}
