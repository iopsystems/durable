use durable::sqlx;
use durable::sqlx::types::Json;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Data {
    a: String,
    b: u32,
}

fn main() -> anyhow::Result<()> {
    sqlx::transaction("create a table with a json column", |mut conn| {
        sqlx::query(
            "
                CREATE TABLE test(
                    id   int    NOT NULL PRIMARY KEY,
                    data jsonb  NOT NULL
                );
                ",
        )
        .execute(&mut conn)
    })?;

    sqlx::transaction("insert a json row into the database", |mut conn| {
        sqlx::query("INSERT INTO test(id, data) VALUES ($1, $2)")
            .bind(1)
            .bind(Json(Data {
                a: "thing 1".into(),
                b: 77,
            }))
            .execute(&mut conn)
    })?;

    let (id, data) = sqlx::transaction("read the row back", |mut conn| {
        sqlx::query("SELECT * FROM test")
            .map(|row| -> (i32, Json<Data>) { (row.get("id"), row.get("data")) })
            .fetch_one(&mut conn)
    })?;

    assert_eq!(id, 1);
    assert_eq!(data.a, "thing 1");
    assert_eq!(data.b, 77);

    Ok(())
}
