//! Make SQL queries as part of your workflow.
//!
//! This module provides a [SQLx](sqlx) driver and wrappers that allow you to
//! interact with the database that the durable runtime is running on. As
//! durable is designed to share a database with your application, this allows
//! you to make all SQL transactions you wish to perform.
//!
//! # Quickstart
//! In order to get a database connection you need to enter a database
//! transaction. You do this by calling the [`transaction`] function:
//!
//! ```
//! use durable::sqlx::transaction;
//!
//! transaction(
//!     "do the thing with the database",
//!     |conn| -> durable::Result<()> {
//!         sqlx::query("INSERT INTO foo(id) VALUES ($1)")
//!             .bind(7)
//!             .execute(&mut *conn)?;
//!
//!         Ok(())
//!     },
//! );
//! ```

use driver::{Durable, QueryResult, Row};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::error::BoxDynError;

pub mod driver;
mod util;

mod bindings {
    #![allow(unused_braces)]

    #[cfg(feature = "bindgen")]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    #[cfg(not(feature = "bindgen"))]
    include!("bindings.rs");

    pub use self::durable::core::sql::*;
}

pub use sqlx::{Error, Result};

#[doc(inline)]
pub use crate::driver::Connection;
use crate::util::BlockingStream;

/// Run a durable transaction with a database transaction.
///
/// This runs a regular durable transaction, but does it all within a single
/// underlying database transaction. At the same time, it makes the database
/// transaction available to the workflow. This means that as long as your
/// workflow is only making changes to the database, you can get exactly-once
/// semantics if the workflow gets interrupted partway through the transaction.
///
/// # Restrictions
/// * Attempting to enter a new transaction within another one will result in
///   the workflow being killed immediately via a trap.
/// * Smuggling the [`Connection`] outside of the connection and attempting to
///   use it will also result in the workflow being killed immediately via a
///   trap.
pub fn transaction<F, T>(label: &str, func: F) -> T
where
    F: Fn(Connection) -> T,
    T: Serialize + DeserializeOwned,
{
    use durable_core::transaction::{transaction_with, TransactionOptions};

    let options = TransactionOptions::new(label).database(true);
    transaction_with(options, || func(Connection::new()))
}

/// Execute a single SQL query as a prepared statement.
///
/// The query string may only contain a single DML statement: `SELECT`,
/// `INSERT`, `UPDATE`, `DELETE`, and variants.
///
/// The durable runtime will transparently prepare and cache the statement,
/// which means it only needs to be parsed once in the connection's lifetime,
/// and any generated query plans can be retained.
///
/// See the [`Query`] type for methods you may call.
///
/// ### Dynamic input: Use Query Parameters (Prevents SQL Injection)
/// At some point, you'll likely want to include some form of dynamic input in
/// your query, possibly from the user.
/// ///
/// Your first instinct might be to do something like this:
/// ```rust,no_run
/// # async fn example() -> durable::sqlx::Result<()> {
/// # let mut conn: durable::sqlx::Connection = unimplemented!();
/// // Imagine this is input from the user, e.g. a search form on a website.
/// let user_input = "possibly untrustworthy input!";
///
/// // DO NOT DO THIS unless you're ABSOLUTELY CERTAIN it's what you need!
/// let query = format!("SELECT * FROM articles WHERE content LIKE '%{user_input}%'");
/// // where `conn` is some type that implements `Executor`.
/// let results = durable::sqlx::query(&query).fetch_all(&mut conn)?;
/// # Ok(())
/// # }
/// ```
///
/// The example above showcases a **SQL injection vulnerability**, because it's
/// trivial for a malicious user to craft an input that can "break out" of the
/// string literal.
///
/// For example, if they send the input `foo'; DELETE FROM articles; --`
/// then your application would send the following to the database server (line
/// breaks added for clarity):
///
/// ```sql
/// SELECT * FROM articles WHERE content LIKE '%foo';
/// DELETE FROM articles;
/// --%'
/// ```
///
/// In this case, because this interface *always* uses prepared statements, you
/// would likely be fine because prepared statements _generally_ (see above) are
/// only allowed to contain a single query. This would simply return an error.
///
/// However, it would also break on legitimate user input.
/// What if someone wanted to search for the string `Alice's Apples`? It would
/// also return an error because the database would receive a query with a
/// broken string literal (line breaks added for clarity):
///
/// ```sql
/// SELECT * FROM articles WHERE content LIKE '%Alice'
/// s Apples%'
/// ```
///
/// Of course, it's possible to make this syntactically valid by escaping the
/// apostrophe, but there's a better way.
///
/// ##### You should always prefer query parameters for dynamic input.
///
/// When using query parameters, you add placeholders to your query where a
/// value should be substituted at execution time, then call
/// [`.bind()`][Query::bind] with that value.
///
/// The syntax for placeholders is unfortunately not standardized and depends on
/// the database. Durable is built on top of Postgres, which uses numbered
/// parameters: `$1`, `$2`, `$3`, etc.
/// * The number is the Nth bound variable, starting from one.
/// * The same placeholder can be used arbitrarily many times to refer to the
///   same bound variable.
///
/// In both cases, the placeholder syntax acts as a variable expression
/// representing the bound value:
///
/// ```rust,no_run
/// # fn example2() -> sqlx::Result<()> {
/// # let mut conn: durable::sqlx::Connection = unimplemented!();
/// let user_input = "Alice's Apples";
///
/// let results = durable::sqlx::query(
///     // Notice how we only have to bind the argument once and we can use it multiple times:
///     "SELECT * FROM articles
///      WHERE title LIKE '%' || $1 || '%'
///      OR content LIKE '%' || $1 || '%'",
/// )
/// .bind(user_input)
/// .fetch_all(&mut conn)?;
/// # Ok(())
/// # }
/// ```
/// ##### The value bound to a query parameter is entirely separate from the query and does not affect its syntax.
/// Thus, SQL injection is impossible (barring shenanigans like calling a SQL
/// function that lets you execute a string as a statement) and *all* strings
/// are valid.
///
/// This also means you cannot use query parameters to add conditional SQL
/// fragments.
///
/// **SQLx does not substitute placeholders on the client side**. It is done by
/// the database server itself.
///
/// ##### SQLx supports many different types for parameter binding, not just strings.
/// Any type that implements [`Encode<DB>`][sqlx::Encode] and
/// [`Type<DB>`][sqlx::Type] can be bound as a parameter.
///
/// See [the `types` module][sqlx::types] (links to `sqlx_core::types` but you
/// should use `sqlx::types`) for details.
///
/// As an additional benefit, query parameters are usually sent in a compact
/// binary encoding instead of a human-readable text encoding, which saves
/// bandwidth.
pub fn query(sql: &str) -> Query<driver::Arguments> {
    Query(sqlx::query(sql))
}

/// Execute a SQL query as a prepared statement (transparently cached), with the
/// given arguments.
///
/// See [`query()`][query] for details, such as supported syntax.
pub fn query_with<'q, A>(sql: &'q str, arguments: A) -> Query<'q, A>
where
    A: sqlx::IntoArguments<'q, Durable>,
{
    Query(sqlx::query_with(sql, arguments))
}

pub fn query_as<'q, O>(sql: &'q str) -> QueryAs<'q, O, driver::Arguments>
where
    O: for<'r> sqlx::FromRow<'r, Row>,
{
    QueryAs(sqlx::query_as(sql))
}

pub fn query_as_with<'q, O, A>(sql: &'q str, arguments: A) -> QueryAs<'q, O, A>
where
    A: sqlx::IntoArguments<'q, Durable>,
    O: for<'r> sqlx::FromRow<'r, Row>,
{
    QueryAs(sqlx::query_as_with(sql, arguments))
}

/// A sincle SQL query as a prepared statement. Returned by [`query()`].
#[must_use = "query must be executed to affect database"]
pub struct Query<'q, A>(sqlx::query::Query<'q, Durable, A>);

impl<'q> Query<'q, driver::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// If the number of times this is called does not match the number of bind
    /// parameters that appear in the query (`$1 .. $N`) then an error will be
    /// returned when this query is executed.
    ///
    /// There is no validation that the value is of the type expected by the
    /// query.
    ///
    /// If encoding the value fails, the error is stored and later surfaced when
    /// executing the query.
    pub fn bind<T>(self, value: T) -> Self
    where
        T: sqlx::Encode<'q, Durable> + sqlx::Type<Durable> + 'q,
    {
        Self(self.0.bind(value))
    }

    /// Like [`Query::bind`] but immediately returns an error if encoding the
    /// value failed.
    pub fn try_bind<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: sqlx::Encode<'q, Durable> + sqlx::Type<Durable> + 'q,
    {
        self.0.try_bind(value)
    }
}

impl<'q, A> Query<'q, A>
where
    A: sqlx::IntoArguments<'q, Durable> + Send + 'q,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`] for a fallible version of this method.
    ///
    /// [`try_map`]: Query::try_map
    pub fn map<F, O>(self, mut f: F) -> Map<'q, impl FnMut(Row) -> Result<O, sqlx::Error> + Send, A>
    where
        F: FnMut(Row) -> O + Send,
        O: Unpin,
    {
        self.try_map(move |row| Ok(f(row)))
    }

    /// Map each row in the result to another type.
    pub fn try_map<F, O>(self, f: F) -> Map<'q, F, A>
    where
        F: FnMut(Row) -> Result<O, sqlx::Error> + Send,
        O: Unpin,
    {
        Map(self.0.try_map(f))
    }

    /// Execute the query and return the total number of rows affected.
    pub fn execute<'e, 'c: 'e, E>(self, executor: E) -> Result<QueryResult, sqlx::Error>
    where
        'q: 'e,
        A: 'e,
        E: sqlx::Executor<'c, Database = Durable>,
    {
        crate::util::block_on(self.0.execute(executor))
    }

    /// Execute the query and return the generated results as an iterator.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> impl Iterator<Item = Result<Row, Error>> + 'e
    where
        'q: 'e,
        A: 'e,
        E: sqlx::Executor<'c, Database = Durable>,
    {
        BlockingStream::new(self.0.fetch(executor))
    }

    /// Execute the query and return all the resulting rows collected into a
    /// [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into
    /// memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known
    /// upper bound, e.g. using `LIMIT`.
    pub fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<Row>, sqlx::Error>
    where
        'q: 'e,
        A: 'e,
        E: sqlx::Executor<'c, Database = Durable>,
    {
        crate::util::block_on(self.0.fetch_all(executor))
    }

    /// Execute the query, returning the first row or [`Error::RowNotFound`]
    /// otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<Row, Error>
    where
        'q: 'e,
        A: 'e,
        E: sqlx::Executor<'c, Database = Durable>,
    {
        crate::util::block_on(self.0.fetch_one(executor))
    }

    /// Execute the query, returning the first row or `None` otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<Row>, Error>
    where
        'q: 'e,
        A: 'e,
        E: sqlx::Executor<'c, Database = Durable>,
    {
        crate::util::block_on(self.0.fetch_optional(executor))
    }
}

#[must_use = "query must be executed to affect database"]
pub struct QueryAs<'q, O, A>(sqlx::query::QueryAs<'q, Durable, O, A>);

impl<'q, O> QueryAs<'q, O, driver::Arguments> {
    /// Bind a value for use with this SQL query.
    ///
    /// See [`Query::bind`](Query::bind).
    pub fn bind<T>(self, value: T) -> Self
    where
        T: sqlx::Encode<'q, Durable> + sqlx::Type<Durable> + 'q,
    {
        Self(self.0.bind(value))
    }
}

impl<'q, O, A> QueryAs<'q, O, A>
where
    A: sqlx::IntoArguments<'q, Durable> + 'q,
    O: for<'r> sqlx::FromRow<'r, Row> + Send + Unpin,
{
    /// Execute the query and return the generated results as a iterator.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> impl Iterator<Item = Result<O, Error>> + 'e
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        O: 'e,
        A: 'e,
    {
        BlockingStream::new(self.0.fetch(executor))
    }

    /// Execute the query and return all the resulting rows collected into a
    /// [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into
    /// memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known
    /// upper bound, e.g. using `LIMIT`.
    pub fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        O: 'e,
        A: 'e,
    {
        crate::util::block_on(self.0.fetch_all(executor))
    }

    /// Execute the query, returning the first row or [`Error::RowNotFound`]
    /// otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        O: 'e,
        A: 'e,
    {
        crate::util::block_on(self.0.fetch_one(executor))
    }

    /// Execute the query, returning the first row or `None` otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        O: 'e,
        A: 'e,
    {
        crate::util::block_on(self.0.fetch_optional(executor))
    }
}

/// A single SQL query that will map its results to an owned Rust type.
///
/// Executes as a prepared statement.
#[must_use = "query must be executed to affect database"]
pub struct Map<'q, F, A>(sqlx::query::Map<'q, Durable, F, A>);

impl<'q, F, O, A> Map<'q, F, A>
where
    F: FnMut(Row) -> Result<O, Error> + Send,
    O: Send + Unpin,
    A: sqlx::IntoArguments<'q, Durable> + Send + 'q,
{
    /// Map each row in the result to another type.
    ///
    /// See [`try_map`](Map::try_map) for a fallible version of this method.
    pub fn map<G, P>(self, g: G) -> Map<'q, impl FnMut(Row) -> Result<P, Error> + Send, A>
    where
        G: FnMut(O) -> P + Send,
        P: Unpin,
    {
        Map(self.0.map(g))
    }

    /// Map each row in the result to another type.
    pub fn try_map<G, P>(self, g: G) -> Map<'q, impl FnMut(Row) -> Result<P, Error> + Send, A>
    where
        G: FnMut(O) -> Result<P, Error> + Send,
        P: Unpin,
    {
        Map(self.0.try_map(g))
    }

    /// Execute the query and return the generated results as a iterator.
    pub fn fetch<'e, 'c: 'e, E>(self, executor: E) -> impl Iterator<Item = Result<O, Error>> + 'e
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        F: 'e,
        O: 'e,
    {
        BlockingStream::new(self.0.fetch(executor))
    }

    /// Execute the query and return all the resulting rows collected into a
    /// [`Vec`].
    ///
    /// ### Note: beware result set size.
    /// This will attempt to collect the full result set of the query into
    /// memory.
    ///
    /// To avoid exhausting available memory, ensure the result set has a known
    /// upper bound, e.g. using `LIMIT`.
    pub fn fetch_all<'e, 'c: 'e, E>(self, executor: E) -> Result<Vec<O>, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        F: 'e,
        O: 'e,
    {
        crate::util::block_on(self.0.fetch_all(executor))
    }

    /// Execute the query, returning the first row or [`Error::RowNotFound`]
    /// otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_one<'e, 'c: 'e, E>(self, executor: E) -> Result<O, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        F: 'e,
        O: 'e,
    {
        crate::util::block_on(self.0.fetch_one(executor))
    }

    /// Execute the query, returning the first row or `None` otherwise.
    ///
    /// ### Note: for best performance, ensure the query returns at most one row.
    /// Depending on the driver implementation, if your query can return more
    /// than one row, it may lead to wasted CPU time and bandwidth on the
    /// database server.
    ///
    /// Even when the driver implementation takes this into account, ensuring
    /// the query returns at most one row can result in a more optimal query
    /// plan.
    ///
    /// If your query has a `WHERE` clause filtering a unique column by a single
    /// value, you're good.
    ///
    /// Otherwise, you might want to add `LIMIT 1` to your query.
    pub fn fetch_optional<'e, 'c: 'e, E>(self, executor: E) -> Result<Option<O>, Error>
    where
        'q: 'e,
        E: sqlx::Executor<'c, Database = Durable> + 'e,
        F: 'e,
        O: 'e,
    {
        crate::util::block_on(self.0.fetch_optional(executor))
    }
}
