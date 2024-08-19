use crate::bindings as sql;
use crate::driver::{Durable, TypeInfo, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Row {
    columns: Vec<Column>,
    values: Vec<Value>,
}

impl Row {
    pub(crate) fn from_raw(raw: sql::Row) -> Self {
        let (columns, values) = raw
            .columns
            .into_iter()
            .enumerate()
            .map(|(idx, column)| {
                let value = Value(column.value);

                (
                    Column {
                        ordinal: idx,
                        name: column.name,
                        type_info: value.type_info(),
                    },
                    value,
                )
            })
            .collect();

        Self { columns, values }
    }

    /// Returns `true` if this row has no columns.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of columns in this row.
    pub fn len(&self) -> usize {
        sqlx::Row::len(self)
    }

    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name and a `usize`
    /// index can be used to access a column by position.
    ///
    /// # Panics
    ///
    /// Panics if the column does not exist or its value cannot be decoded into
    /// the requested type. See [`try_get`](Self::try_get) for a
    /// non-panicking version.
    pub fn get<'r, T, I>(&'r self, index: I) -> T
    where
        I: sqlx::ColumnIndex<Self>,
        T: sqlx::Decode<'r, Durable> + sqlx::Type<Durable>,
    {
        sqlx::Row::get(self, index)
    }

    /// Index into the database row and decode a single value.
    ///
    /// A string index can be used to access a column by name and a `usize`
    /// index can be used to access a column by position.
    ///
    /// # Panics
    ///
    /// Panics if the column does not exist or its value cannot be decoded into
    /// the requested type. See [`try_get`](Self::try_get) for a
    /// non-panicking version.
    pub fn try_get<'r, T, I>(&'r self, index: I) -> Result<T, sqlx::Error>
    where
        I: sqlx::ColumnIndex<Self>,
        T: sqlx::Decode<'r, Durable> + sqlx::Type<Durable>,
    {
        sqlx::Row::try_get(self, index)
    }
}

impl sqlx::Row for Row {
    type Database = Durable;

    fn columns(&self) -> &[<Self::Database as sqlx::Database>::Column] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<&'_ Value, sqlx::Error>
    where
        I: sqlx::ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        Ok(&self.values[index])
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Column {
    ordinal: usize,
    name: String,
    type_info: TypeInfo,
}

impl sqlx::Column for Column {
    type Database = Durable;

    fn name(&self) -> &str {
        &self.name
    }

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn type_info(&self) -> &TypeInfo {
        &self.type_info
    }
}

impl sqlx::ColumnIndex<Row> for usize {
    fn index(&self, row: &Row) -> Result<usize, sqlx::Error> {
        if *self >= row.len() {
            Err(sqlx::Error::ColumnIndexOutOfBounds {
                index: *self,
                len: row.len(),
            })
        } else {
            Ok(*self)
        }
    }
}

impl sqlx::ColumnIndex<Row> for &str {
    fn index(&self, row: &Row) -> Result<usize, sqlx::Error> {
        row.columns
            .iter()
            .position(|col| col.name == *self)
            .ok_or_else(|| sqlx::Error::ColumnNotFound(self.to_string()))
    }
}
