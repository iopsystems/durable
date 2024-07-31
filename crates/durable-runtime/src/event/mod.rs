use async_trait::async_trait;

mod events;

pub use self::events::*;

#[derive(Clone, Debug)]
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

#[async_trait]
pub trait EventSource: Send {
    async fn next(&mut self) -> anyhow::Result<Event>;
}
