//! Scheduler trait for deterministic simulation testing (DST).
//!
//! The [`Scheduler`] trait allows controlling the interleaving of concurrent
//! worker components. In production, the default [`NoopScheduler`] is used,
//! which never blocks. In DST, a custom scheduler can gate each component
//! behind a permit, allowing the test harness to deterministically control
//! execution order.

use std::borrow::Cow;
use std::fmt;

/// Identifies a worker component that is requesting permission to proceed.
///
/// The scheduler uses this to decide which component to run next.
#[derive(Clone, Debug)]
pub enum Component {
    /// The heartbeat loop is about to update the worker's heartbeat timestamp.
    Heartbeat { worker_id: i64 },

    /// The worker validation loop is about to check for dead workers.
    ValidateWorkers { worker_id: i64 },

    /// The leader loop is about to wake suspended tasks.
    Leader { worker_id: i64 },

    /// The task cleanup loop is about to delete old completed tasks.
    TaskCleanup { worker_id: i64 },

    /// The stuck notification loop is about to unwedge stuck tasks.
    StuckNotify { worker_id: i64 },

    /// The event processing loop is about to handle an event.
    ProcessEvents { worker_id: i64 },

    /// The worker is about to claim and spawn new tasks.
    SpawnTasks { worker_id: i64 },

    /// A task is entering or exiting a transaction with the database.
    TaskTransaction {
        task_id: i64,
        label: Cow<'static, str>,
    },

    /// A user-defined component for downstream DST.
    ///
    /// This allows systems built on top of durable to participate in the
    /// same scheduling framework.
    Custom(Cow<'static, str>),
}

/// Events emitted by the runtime at key state transitions.
///
/// These are delivered to the scheduler via [`Scheduler::notify`] for
/// logging, assertions, and driving simulation decisions.
#[derive(Clone, Debug)]
pub enum ScheduleEvent {
    /// A worker registered itself in the database.
    WorkerRegistered { worker_id: i64 },

    /// A worker was deleted from the database (normal shutdown or expiry).
    WorkerDeleted { worker_id: i64 },

    /// The cluster leader changed.
    LeaderChanged { new_leader: i64 },

    /// A task was claimed by a worker.
    TaskClaimed { worker_id: i64, task_id: i64 },

    /// A task completed (successfully or with failure).
    TaskCompleted { task_id: i64, success: bool },

    /// A task was suspended.
    TaskSuspended {
        task_id: i64,
        wakeup_at: Option<chrono::DateTime<chrono::Utc>>,
    },

    /// A task's transaction was recorded to the event log.
    TransactionRecorded {
        task_id: i64,
        index: i32,
        label: Cow<'static, str>,
    },

    /// Suspended tasks were woken by the leader.
    TasksWoken { count: u64 },
}

/// RAII guard returned by [`Scheduler::acquire`].
///
/// While this guard is held, the scheduler knows the component is actively
/// executing. When dropped, the scheduler is notified that the step completed.
///
/// The default implementation is a no-op. Custom schedulers can use the
/// [`on_drop`](ScheduleGuard::on_drop) constructor to attach cleanup logic.
pub struct ScheduleGuard {
    _inner: Option<Box<dyn FnOnce() + Send>>,
}

impl ScheduleGuard {
    /// Create a no-op guard.
    pub fn noop() -> Self {
        Self { _inner: None }
    }

    /// Create a guard that calls `f` when dropped.
    pub fn on_drop(f: impl FnOnce() + Send + 'static) -> Self {
        Self {
            _inner: Some(Box::new(f)),
        }
    }
}

impl Drop for ScheduleGuard {
    fn drop(&mut self) {
        if let Some(f) = self._inner.take() {
            f();
        }
    }
}

impl fmt::Debug for ScheduleGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScheduleGuard")
            .field("has_callback", &self._inner.is_some())
            .finish()
    }
}

/// Controls the execution order of worker components.
///
/// In production, the default [`NoopScheduler`] is used, which never blocks
/// and ignores all events. In deterministic simulation testing, a custom
/// scheduler can gate each component behind a permit, allowing the test
/// harness to control interleaving.
///
/// # Usage for DST
///
/// A DST scheduler would typically:
/// 1. Collect pending `acquire()` calls from all components
/// 2. Use a seeded RNG to pick which component runs next
/// 3. Record all `notify()` events for post-test assertions
///
/// # Usage for downstream systems
///
/// Systems built on top of durable can share the same scheduler by using
/// [`Component::Custom`] to register their own components.
#[async_trait::async_trait]
pub trait Scheduler: Send + Sync {
    /// Called by a component before it does work.
    ///
    /// The scheduler can delay this call arbitrarily to control ordering.
    /// Returns a guard that is held while the component does its work.
    /// Dropping the guard signals that the step completed.
    async fn acquire(&self, component: Component) -> ScheduleGuard;

    /// Notify the scheduler about a runtime state transition.
    ///
    /// This is informational — the scheduler can use it for logging,
    /// assertions, or to inform future scheduling decisions.
    fn notify(&self, event: ScheduleEvent);
}

/// A no-op scheduler that never blocks and ignores all events.
///
/// This is the default scheduler used in production.
pub struct NoopScheduler;

#[async_trait::async_trait]
impl Scheduler for NoopScheduler {
    async fn acquire(&self, _component: Component) -> ScheduleGuard {
        ScheduleGuard::noop()
    }

    fn notify(&self, _event: ScheduleEvent) {}
}
