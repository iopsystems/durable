use std::net::IpAddr;
use std::str::FromStr;

use durable::sqlx;

fn main() -> anyhow::Result<()> {
    let addr = IpAddr::from_str("127.0.0.1")?;

    sqlx::transaction("create the test table", |mut conn| {
        sqlx::query("CREATE TABLE test(addr inet)").execute(&mut conn)
    })?;

    sqlx::transaction("insert data", |mut conn| {
        sqlx::query("INSERT INTO test(addr) VALUES ($1)")
            .bind(addr)
            .execute(&mut conn)
    })?;

    let fetched = sqlx::transaction("fetch data", |mut conn| -> Result<IpAddr, _> {
        sqlx::query_scalar("SELECT addr FROM test").fetch_one(&mut conn)
    })?;

    assert_eq!(addr, fetched);

    Ok(())
}
