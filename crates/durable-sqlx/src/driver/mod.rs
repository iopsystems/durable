//! The sqlx driver for interacting with the database.
//!
//! You will usually not need to interact with these, but they are here if
//! necessary. They are all async to work with sqlx, though making the actual
//! sql query is synchronous on the backend.

mod arguments;
mod connection;
mod database;
mod error;
mod row;
mod statement;
mod transaction;
mod type_info;
mod types;
mod value;

pub use self::arguments::Arguments;
pub use self::connection::{ConnectOptions, Connection};
pub use self::database::{Durable, QueryResult};
pub(crate) use self::error::DatabaseError;
pub use self::row::{Column, Row};
pub use self::statement::Statement;
pub use self::transaction::TransactionManager;
pub use self::type_info::TypeInfo;
pub use self::value::Value;
