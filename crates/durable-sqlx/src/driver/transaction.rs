use sqlx::{AssertSqlSafe, SqlStr};

use crate::driver::{Connection, Durable};

pub enum TransactionManager {}

impl TransactionManager {
    async fn begin(conn: &mut Connection) -> Result<(), sqlx::Error> {
        let sql = format!("SAVEPOINT savepoint_{}", conn.txn_depth);
        sqlx::query(AssertSqlSafe(sql)).execute(&mut *conn).await?;
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
        sqlx::query(AssertSqlSafe(sql)).execute(&mut *conn).await?;
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
        sqlx::query(AssertSqlSafe(sql)).execute(&mut *conn).await?;
        conn.txn_depth -= 1;

        Ok(())
    }
}

impl sqlx_core::transaction::TransactionManager for TransactionManager {
    type Database = Durable;

    async fn begin(conn: &mut Connection, _statement: Option<SqlStr>) -> Result<(), sqlx::Error> {
        Self::begin(conn).await
    }

    async fn commit(conn: &mut Connection) -> Result<(), sqlx::Error> {
        Self::commit(conn).await
    }

    async fn rollback(conn: &mut Connection) -> Result<(), sqlx::Error> {
        Self::rollback(conn).await
    }

    fn start_rollback(conn: &mut Connection) {
        let _ = crate::util::block_on(Self::rollback(conn));
    }

    fn get_transaction_depth(conn: &Connection) -> usize {
        conn.txn_depth as usize
    }
}
