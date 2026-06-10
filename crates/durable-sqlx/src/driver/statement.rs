use sqlx::SqlStr;

use crate::driver::Durable;

#[derive(Clone)]
pub struct Statement(SqlStr);

impl Statement {
    pub(crate) fn new(sql: SqlStr) -> Self {
        Self(sql)
    }
}

impl sqlx::Statement for Statement {
    type Database = Durable;

    fn into_sql(self) -> SqlStr {
        self.0
    }

    fn sql(&self) -> &SqlStr {
        &self.0
    }

    fn parameters(
        &self,
    ) -> Option<sqlx::Either<&[<Self::Database as sqlx::Database>::TypeInfo], usize>> {
        None
    }

    fn columns(&self) -> &[<Self::Database as sqlx::Database>::Column] {
        &[]
    }

    fn query(
        &self,
    ) -> sqlx::query::Query<'_, Self::Database, <Self::Database as sqlx::Database>::Arguments> {
        sqlx_core::query::query_statement(self)
    }

    fn query_with<A>(&self, arguments: A) -> sqlx::query::Query<'_, Self::Database, A>
    where
        A: sqlx::IntoArguments<Self::Database>,
    {
        sqlx_core::query::query_statement_with(self, arguments)
    }

    fn query_as<O>(
        &self,
    ) -> sqlx::query::QueryAs<'_, Self::Database, O, <Self::Database as sqlx::Database>::Arguments>
    where
        O: for<'r> sqlx::FromRow<'r, <Self::Database as sqlx::Database>::Row>,
    {
        sqlx_core::query_as::query_statement_as(self)
    }

    fn query_as_with<'s, O, A>(
        &'s self,
        arguments: A,
    ) -> sqlx::query::QueryAs<'s, Self::Database, O, A>
    where
        O: for<'r> sqlx::FromRow<'r, <Self::Database as sqlx::Database>::Row>,
        A: sqlx::IntoArguments<Self::Database>,
    {
        sqlx_core::query_as::query_statement_as_with(self, arguments)
    }

    fn query_scalar<O>(
        &self,
    ) -> sqlx::query::QueryScalar<
        '_,
        Self::Database,
        O,
        <Self::Database as sqlx::Database>::Arguments,
    >
    where
        (O,): for<'r> sqlx::FromRow<'r, <Self::Database as sqlx::Database>::Row>,
    {
        sqlx_core::query_scalar::query_statement_scalar(self)
    }

    fn query_scalar_with<'s, O, A>(
        &'s self,
        arguments: A,
    ) -> sqlx::query::QueryScalar<'s, Self::Database, O, A>
    where
        (O,): for<'r> sqlx::FromRow<'r, <Self::Database as sqlx::Database>::Row>,
        A: sqlx::IntoArguments<Self::Database>,
    {
        sqlx_core::query_scalar::query_statement_scalar_with(self, arguments)
    }
}
