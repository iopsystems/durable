use crate::driver::{
    Arguments, Column, Connection, Row, Statement, TransactionManager, TypeInfo, Value,
};

#[derive(Debug)]
pub struct Durable;

impl sqlx::Database for Durable {
    type Connection = Connection;
    type TransactionManager = TransactionManager;
    type Row = Row;
    type QueryResult = QueryResult;
    type Column = Column;
    type TypeInfo = TypeInfo;
    type Value = Value;
    type ValueRef<'r> = &'r Value;
    type Arguments<'q> = Arguments;
    type ArgumentBuffer<'q> = Vec<Value>;
    type Statement<'q> = Statement<'q>;

    const NAME: &'static str = "durable";
    const URL_SCHEMES: &'static [&'static str] = &[];
}

#[derive(Copy, Clone, Default, Debug)]
pub struct QueryResult {
    rows_affected: u64,
}

impl QueryResult {
    pub(crate) fn new(rows_affected: u64) -> Self {
        Self { rows_affected }
    }

    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<QueryResult> for QueryResult {
    fn extend<T: IntoIterator<Item = QueryResult>>(&mut self, iter: T) {
        for item in iter {
            self.rows_affected += item.rows_affected;
        }
    }
}
