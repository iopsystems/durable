use std::sync::OnceLock;

use durable::sqlx::driver::{Durable, TypeInfo, Value};

#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
enum TestDummy {
    A,
    B,
    C,
    Blargh,
}

impl sqlx::Encode<'_, Durable> for TestDummy {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let tyinfo = <Self as sqlx::Type<Durable>>::type_info();
        let value = match self {
            Self::A => "a",
            Self::B => "b",
            Self::C => "c",
            Self::Blargh => "blargh",
        };

        buf.push(Value::enum_scalar(value, &tyinfo));
        Ok(sqlx::encode::IsNull::No)
    }
}

impl sqlx::Type<Durable> for TestDummy {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        static TYINFO_CACHE: OnceLock<TypeInfo> = OnceLock::new();

        // TypeInfo::with_name currently involves a database query within the runtime.
        // This is expensive, so only do it once and then hand out copies.
        TYINFO_CACHE
            .get_or_init(|| {
                TypeInfo::with_name("test_dummy")
                    .expect("test_dummy type was not present within the database")
            })
            .clone()
    }
}

fn main() -> anyhow::Result<()> {
    durable::sqlx::transaction("set up the database schema", |mut conn| {
        durable::sqlx::query("CREATE TYPE test_dummy AS ENUM('a', 'b', 'c', 'blargh')")
            .execute(&mut conn)?;

        durable::sqlx::query("CREATE TABLE test(id bigint, value test_dummy)").execute(&mut conn)
    })?;

    durable::sqlx::transaction("insert into the test table", |mut conn| {
        durable::sqlx::query("INSERT INTO test(id, value) VALUES(1, $1)")
            .bind(TestDummy::Blargh)
            .execute(&mut conn)
    })?;

    Ok(())
}
