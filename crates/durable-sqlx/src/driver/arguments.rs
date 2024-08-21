use crate::bindings as sql;
use crate::driver::{Durable, Value};

#[derive(Default)]
pub struct Arguments(Vec<Value>);

impl Arguments {
    pub(crate) fn into_raw_args(self) -> Vec<sql::Value> {
        // SAFETY: Value is a #[repr(transparent)] wrapper around sql::Value.
        unsafe { std::mem::transmute(self.0) }
    }
}

impl<'q> sqlx::Arguments<'q> for Arguments {
    type Database = Durable;

    fn reserve(&mut self, additional: usize, _: usize) {
        self.0.reserve(additional);
    }

    fn add<T>(&mut self, value: T) -> Result<(), sqlx::error::BoxDynError>
    where
        T: 'q + sqlx::Encode<'q, Self::Database> + sqlx::Type<Self::Database>,
    {
        let ty = value.produces().unwrap_or_else(T::type_info);
        if value.encode(&mut self.0)?.is_null() {
            self.0.push(Value::new(sql::Value::null(ty.into_inner())));
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'q> sqlx::IntoArguments<'q, Durable> for Arguments {
    fn into_arguments(self) -> <Durable as sqlx::Database>::Arguments<'q> {
        self
    }
}
