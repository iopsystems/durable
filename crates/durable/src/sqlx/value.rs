use chrono::DateTime;


pub enum Value {
    Null,
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    String(String),
    Bytea(Vec<u8>),
    // TimestampTz(DateTime<>)
}


