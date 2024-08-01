use serde::{Deserialize, Serialize};

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
