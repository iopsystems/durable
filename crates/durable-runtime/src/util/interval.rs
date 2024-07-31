use sqlx::postgres::types::PgInterval;

pub(crate) trait IntoPgInterval {
    fn into_pg_interval(self) -> PgInterval;
}

impl IntoPgInterval for std::time::Duration {
    fn into_pg_interval(self) -> PgInterval {
        PgInterval {
            months: 0,
            days: 0,
            microseconds: self
                .as_nanos()
                .try_into()
                .expect("duration was longer than can be represented in a PgInterval"),
        }
    }
}
