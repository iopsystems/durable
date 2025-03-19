use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;

use crate::Durable;

impl<'r, T> sqlx::Encode<'r, Durable> for Option<T>
where
    T: sqlx::Encode<'r, Durable>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'r>,
    ) -> Result<IsNull, BoxDynError> {
        match self {
            Some(value) => value.encode_by_ref(buf),
            None => Ok(IsNull::Yes),
        }
    }
}
