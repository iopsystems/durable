use futures_core::future::BoxFuture;

use crate::driver::{Connection, Durable};

pub enum TransactionManager {}

impl TransactionManager {
    async fn begin(conn: &mut Connection) -> Result<(), sqlx::Error> {
        let sql = format!("SAVEPOINT savepoint_{}", conn.txn_depth);
        sqlx::query(&sql).execute(&mut *conn).await?;
        conn.txn_depth += 1;

        Ok(())
    }

    async fn commit(conn: &mut Connection) -> Result<(), sqlx::Error> {
        if conn.txn_depth == 0 {
            return Err(sqlx::Error::Protocol(
                "attempted to commit a database transaction when no transaction was active".into(),
            ));
        }

        let sql = format!("RELEASE savepoint_{}", conn.txn_depth - 1);
        sqlx::query(&sql).execute(&mut *conn).await?;
        conn.txn_depth -= 1;

        Ok(())
    }

    async fn rollback(conn: &mut Connection) -> Result<(), sqlx::Error> {
        if conn.txn_depth == 0 {
            return Err(sqlx::Error::Protocol(
                "attempted to rollback a database transaction when no transaction was active"
                    .into(),
            ));
        }

        let sql = format!("ROLLBACK TO savepoint_{}", conn.txn_depth - 1);
        sqlx::query(&sql).execute(&mut *conn).await?;
        conn.txn_depth -= 1;

        Ok(())
    }
}

impl sqlx::TransactionManager for TransactionManager {
    type Database = Durable;

    fn begin(conn: &mut Connection) -> BoxFuture<'_, Result<(), sqlx::Error>> {
        Box::pin(Self::begin(conn))
    }

    fn commit(conn: &mut Connection) -> BoxFuture<'_, Result<(), sqlx::Error>> {
        Box::pin(Self::commit(conn))
    }

    fn rollback(conn: &mut Connection) -> BoxFuture<'_, Result<(), sqlx::Error>> {
        Box::pin(Self::rollback(conn))
    }

    fn start_rollback(conn: &mut Connection) {
        let _ = crate::util::block_on(Self::rollback(conn));
    }
}
