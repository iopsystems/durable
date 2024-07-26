use async_trait::async_trait;

pub enum Event {
    TaskInserted(TaskInserted),
    NotificationInserted(NotificationInserted),

    /// This event should be emitted whenever there is a possibility that an
    /// event was lost (even if it is not known for sure).
    Lagged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskInserted {
    pub id: i64,
    #[serde(default)]
    pub running_on: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationInserted {
    pub task_id: i64,
    pub event: String,
}

#[async_trait]
pub trait EventSource: Send {
    async fn next(&mut self) -> anyhow::Result<Event>;
}
