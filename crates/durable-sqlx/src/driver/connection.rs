use std::str::FromStr;
use std::time::Duration;

use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use log::LevelFilter;
use url::Url;

use crate::bindings as sql;
use crate::driver::error::{convert_query_error, UnsupportedError};
use crate::driver::{Arguments, Durable, QueryResult, Row, Statement, TypeInfo};

#[derive(Debug, Clone)]
pub enum ConnectOptions {}

impl sqlx::ConnectOptions for ConnectOptions {
    type Connection = Connection;

    fn from_url(url: &Url) -> Result<Self, sqlx::Error> {
        Self::from_str(url.as_str())
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, sqlx::Error>> {
        match *self {}
    }

    fn log_statements(self, _: LevelFilter) -> Self {
        match self {}
    }

    fn log_slow_statements(self, _: LevelFilter, _: Duration) -> Self {
        match self {}
    }

    fn to_url_lossy(&self) -> Url {
        match *self {}
    }

    fn disable_statement_logging(self) -> Self {
        match self {}
    }
}

impl FromStr for ConnectOptions {
    type Err = sqlx::Error;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err(sqlx::Error::Configuration(
            Box::new(UnsupportedError::new()),
        ))
    }
}

/// A connection to the workflow's environment database connection.
///
/// In a durable workflow the database connection is implicitly global and can
/// only be created as part of a transaction. As such, this type is effectively
/// just a permission token that allows you to make database calls.
///
/// Many of the operations within the connection are effectively no-ops:
/// - Closing the connection does nothing. As the transaction is managed by the
///   runtime we don't actually have the permissions necessary to close it.
/// - The same applies for `ping`ing the connection.
#[derive(Debug)]
pub struct Connection {
    /// The nested savepoint depth that we are at.
    pub(crate) txn_depth: u32,
}

impl Connection {
    /// Construct a new connection.
    ///
    /// This connection object refers to the ambient runtime connection. There
    /// can only be one so care must be taken to ensure that there is only ever
    /// one live `Connection` object.
    pub(crate) fn new() -> Self {
        Self { txn_depth: 0 }
    }

    fn run(&mut self, sql: &str, arguments: Arguments, options: sql::Options) -> QueryIterator {
        let params = arguments.raw_args();
        sql::query(sql, params, options);

        QueryIterator
    }
}

impl sqlx::Connection for Connection {
    type Database = Durable;
    type Options = ConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), sqlx::Error>> {
        Box::pin(std::future::ready(Ok(())))
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), sqlx::Error>> {
        self.close()
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), sqlx::Error>> {
        Box::pin(std::future::ready(Ok(())))
    }

    fn begin(
        &mut self,
    ) -> BoxFuture<'_, Result<sqlx::Transaction<'_, Self::Database>, sqlx::Error>> {
        sqlx::Transaction::begin(self)
    }

    fn shrink_buffers(&mut self) {}

    fn flush(&mut self) -> BoxFuture<'_, Result<(), sqlx::Error>> {
        Box::pin(std::future::ready(Ok(())))
    }

    fn should_flush(&self) -> bool {
        false
    }
}

impl<'c> sqlx::Acquire<'c> for &'c mut Connection {
    type Database = Durable;
    type Connection = &'c mut Connection;

    fn acquire(self) -> BoxFuture<'c, Result<Self::Connection, sqlx::Error>> {
        Box::pin(std::future::ready(Ok(self)))
    }

    fn begin(self) -> BoxFuture<'c, Result<sqlx::Transaction<'c, Self::Database>, sqlx::Error>> {
        <Connection as sqlx::Connection>::begin(self)
    }
}

impl<'c> sqlx::Executor<'c> for &'c mut Connection {
    type Database = Durable;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxStream<
        'e,
        Result<sqlx::Either<<Self::Database as sqlx::Database>::QueryResult, Row>, sqlx::Error>,
    >
    where
        'c: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        use async_stream::try_stream;

        let sql = query.sql();
        let params = query.take_arguments().map_err(sqlx::Error::Encode);
        let options = sql::Options {
            limit: u8::MAX,
            persistent: true,
        };

        Box::pin(try_stream! {
            let params = params?.unwrap_or_default();
            let iter = self.run(sql, params, options);

            for item in iter {
                yield item?;
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxFuture<'e, Result<Option<Row>, sqlx::Error>>
    where
        'c: 'e,
        E: 'q + sqlx::Execute<'q, Self::Database>,
    {
        let sql = query.sql();
        let params = query.take_arguments().map_err(sqlx::Error::Encode);
        let options = sql::Options {
            limit: 1,
            persistent: true,
        };

        Box::pin(async move {
            let params = params?.unwrap_or_default();
            let iter = self.run(sql, params, options);

            for item in iter {
                if let sqlx::Either::Right(row) = item? {
                    return Ok(Some(row));
                }
            }

            Ok(None)
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _: &'e [TypeInfo],
    ) -> BoxFuture<'e, Result<Statement<'q>, sqlx::Error>>
    where
        'c: 'e,
    {
        Box::pin(std::future::ready(Ok(Statement::new(sql))))
    }

    fn describe<'e, 'q: 'e>(
        self,
        _: &'q str,
    ) -> BoxFuture<'e, Result<sqlx::Describe<Self::Database>, sqlx::Error>>
    where
        'c: 'e,
    {
        unimplemented!("describe is not implemented for the durable driver")
    }
}

struct QueryIterator;

impl Iterator for QueryIterator {
    type Item = Result<sqlx::Either<QueryResult, Row>, sqlx::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match sql::fetch()? {
            Ok(sql::QueryResult::Count(count)) => Ok(sqlx::Either::Left(QueryResult::new(count))),
            Ok(sql::QueryResult::Row(row)) => Ok(sqlx::Either::Right(Row::from_raw(row))),
            Err(e) => Err(convert_query_error(e)),
        })
    }
}
