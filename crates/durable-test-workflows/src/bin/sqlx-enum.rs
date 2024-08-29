use durable::sqlx::driver::{Durable, TypeInfo};

#[derive(Copy, Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
enum TaskState {
    Ready,
    Active,
    Suspended,
    Complete,
    Failed,
}

impl sqlx::Encode<'_, Durable> for TaskState {
    fn encode_by_ref(
        &self,
        buf: &mut <Durable as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = match self {
            Self::Ready => "ready",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Complete => "complete",
            Self::Failed => "failed",
        };

        <&str as sqlx::Encode<Durable>>::encode(value, buf)
    }
}

impl sqlx::Decode<'_, Durable> for TaskState {
    fn decode(
        value: <Durable as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <String as sqlx::Decode<Durable>>::decode(value)?;

        Ok(match &*value {
            "ready" => Self::Ready,
            "active" => Self::Active,
            "suspended" => Self::Suspended,
            "complete" => Self::Complete,
            "failed" => Self::Failed,
            _ => return Err(format!("invalid durable.task_state enum value `{value}`").into()),
        })
    }
}

impl sqlx::Type<Durable> for TaskState {
    fn type_info() -> <Durable as sqlx::Database>::TypeInfo {
        TypeInfo::with_name("durable.task_state")
            .expect("durable.task_state was not present within the database")
    }
}

fn main() -> anyhow::Result<()> {
    let task = durable::task();
    let state = durable::sqlx::transaction("get the current task state", |mut conn| {
        durable::sqlx::query_scalar!(
            r#"
            SELECT state as "state!: TaskState"
             FROM durable.task
            WHERE id = $1
            "#,
            task.id()
        )
        .fetch_one(&mut conn)
    })?;

    assert_eq!(state, TaskState::Active);

    Ok(())
}
