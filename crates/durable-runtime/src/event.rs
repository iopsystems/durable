use async_trait::async_trait;

pub enum Event {
    /// A `durable:task` event was emitted.
    ///
    /// This occurs in two cases:
    /// * A new task is inserted into the database.
    /// * An existing task in the 'active' state has its runner_id set to NULL.
    Task(Task),

    TaskSuspend(TaskSuspend),

    /// A `durable:notification` event was emitted.
    ///
    /// This occurs when a new entry is inserted into the `notification` table.
    Notification(Notification),

    /// A `durable:worker` event was emitted.
    ///
    /// This occurs when an entry is inserted into or deleted from the `worker`
    /// table. It is used by the worker to determine whether it is a leader.
    Worker(Worker),

    /// This event should be emitted whenever there is a possibility that an
    /// event was lost (even if it is not known for sure).
    Lagged,
}

/// A task event.
///
/// This is emitted when a new task is inserted or an existing task becomes
/// ready to be scheduled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: i64,
    #[serde(default)]
    pub running_on: Option<i64>,
}

/// A task suspend event.
///
/// This is emitted when a new task is inserted or an existing task becomes
/// ready to be scheduled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskSuspend {
    pub id: i64,
}

/// A task-complete event.
///
/// This is emitted when a task transitions to a final state.
///
/// It is not used by the worker but is instead used by the client.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskComplete {
    pub id: i64,
    pub state: TaskCompleteState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskCompleteState {
    Complete,
    Failed,
}

/// A notification event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub task_id: i64,
    pub event: String,
}

/// A log event.
///
/// This is emitted when a new entry is inserted into the `log` table. It is not
/// used by the worker but instead by the client to listen to task logs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Log {
    pub task_id: i64,
    pub index: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Worker {
    pub worker_id: i64,
}

#[async_trait]
pub trait EventSource: Send {
    async fn next(&mut self) -> anyhow::Result<Event>;
}
