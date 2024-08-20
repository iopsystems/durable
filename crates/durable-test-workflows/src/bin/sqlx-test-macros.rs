use durable::sqlx;

#[derive(serde::Serialize, serde::Deserialize)]
struct Record {
    label: String,
    value: String,
}

fn main() -> anyhow::Result<()> {
    let task = durable::task();

    println!("event log 1");
    println!("event log 2");

    let events = sqlx::transaction("get our task data", |mut conn| -> sqlx::Result<_> {
        let events = sqlx::query_as!(
            Record,
            r#"
            SELECT
                label,
                value::text as "value!"
            FROM durable.event
            WHERE task_id = $1
            ORDER BY index ASC
            "#,
            task.id()
        )
        .fetch_all(&mut conn)?;

        Ok(events)
    })?;

    for (idx, event) in events.iter().enumerate() {
        println!("{idx:02}: {} - {}", event.label, event.value);
    }

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].label, "wasi:io/streams.output-stream.write");

    Ok(())
}
