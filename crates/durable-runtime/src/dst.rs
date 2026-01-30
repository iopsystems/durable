//! DST (Deterministic Simulation Testing) implementations.
//!
//! This module provides test implementations of the [`Scheduler`], [`Clock`],
//! and [`Entropy`] traits for use in deterministic simulation testing.
//!
//! # Example
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use durable_runtime::{WorkerBuilder};
//! use durable_runtime::dst::{DstScheduler, DstClock, DstEntropy};
//!
//! # async fn example(pool: sqlx::PgPool) -> anyhow::Result<()> {
//! let seed = 42u64;
//! let scheduler = Arc::new(DstScheduler::new(seed));
//! let clock = Arc::new(DstClock::new(chrono::Utc::now()));
//! let entropy = Arc::new(DstEntropy::new(seed));
//!
//! let worker = WorkerBuilder::new(pool)
//!     .scheduler(scheduler.clone())
//!     .clock(clock.clone())
//!     .entropy(entropy.clone())
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use tokio::sync::Notify;

use crate::clock::Clock;
use crate::entropy::Entropy;
use crate::event::{Event, EventSource};
use crate::scheduler::{Component, ScheduleEvent, ScheduleGuard, Scheduler};

/// A deterministic scheduler that records all acquire/notify calls and allows
/// test harnesses to inspect them.
///
/// In its default mode, it does not block — it behaves like [`NoopScheduler`]
/// but records all events. For full interleaving control, use the gating mode
/// via [`DstScheduler::with_gating`].
///
/// [`NoopScheduler`]: crate::NoopScheduler
pub struct DstScheduler {
    seed: u64,
    events: Mutex<Vec<ScheduleEvent>>,
    acquires: Mutex<Vec<Component>>,
    acquire_count: AtomicU64,
}

impl DstScheduler {
    /// Create a new DST scheduler with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            events: Mutex::new(Vec::new()),
            acquires: Mutex::new(Vec::new()),
            acquire_count: AtomicU64::new(0),
        }
    }

    /// Get the seed used by this scheduler.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Get a snapshot of all recorded events.
    pub fn events(&self) -> Vec<ScheduleEvent> {
        self.events.lock().clone()
    }

    /// Get a snapshot of all recorded acquire calls.
    pub fn acquires(&self) -> Vec<Component> {
        self.acquires.lock().clone()
    }

    /// Get the total number of acquire calls made.
    pub fn acquire_count(&self) -> u64 {
        self.acquire_count.load(Ordering::Relaxed)
    }

    /// Clear all recorded events and acquires.
    pub fn clear(&self) {
        self.events.lock().clear();
        self.acquires.lock().clear();
        self.acquire_count.store(0, Ordering::Relaxed);
    }

    /// Wait until the acquire count reaches at least `n`.
    ///
    /// This is useful for waiting until a certain number of scheduling
    /// points have been reached before performing assertions.
    pub async fn wait_for_acquires(&self, n: u64) {
        loop {
            if self.acquire_count.load(Ordering::Relaxed) >= n {
                return;
            }
            tokio::task::yield_now().await;
        }
    }

    /// Wait until an event matching the predicate is recorded.
    pub async fn wait_for_event<F>(&self, mut pred: F)
    where
        F: FnMut(&ScheduleEvent) -> bool,
    {
        loop {
            {
                let events = self.events.lock();
                if events.iter().any(&mut pred) {
                    return;
                }
            }
            tokio::task::yield_now().await;
        }
    }
}

#[async_trait::async_trait]
impl Scheduler for DstScheduler {
    async fn acquire(&self, component: Component) -> ScheduleGuard {
        self.acquires.lock().push(component);
        self.acquire_count.fetch_add(1, Ordering::Relaxed);
        ScheduleGuard::noop()
    }

    fn notify(&self, event: ScheduleEvent) {
        self.events.lock().push(event);
    }
}

/// A deterministic clock that returns controlled time values.
///
/// Time does not advance automatically — it only changes when explicitly
/// advanced via [`advance`](DstClock::advance) or [`set`](DstClock::set).
///
/// Calls to [`sleep`](Clock::sleep) will complete when the clock is advanced
/// past the sleep deadline.
pub struct DstClock {
    now: Mutex<DateTime<Utc>>,
    notify: Notify,
}

impl DstClock {
    /// Create a new DST clock starting at the given time.
    pub fn new(start: DateTime<Utc>) -> Self {
        Self {
            now: Mutex::new(start),
            notify: Notify::new(),
        }
    }

    /// Advance the clock by the given duration, waking any expired sleepers.
    pub fn advance(&self, duration: Duration) {
        let mut now = self.now.lock();
        *now += duration;
        drop(now);
        self.notify.notify_waiters();
    }

    /// Set the clock to a specific time, waking any expired sleepers.
    pub fn set(&self, time: DateTime<Utc>) {
        *self.now.lock() = time;
        self.notify.notify_waiters();
    }

    /// Get the current simulated time.
    pub fn current_time(&self) -> DateTime<Utc> {
        *self.now.lock()
    }
}

#[async_trait::async_trait]
impl Clock for DstClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.lock()
    }

    fn system_time_now(&self) -> std::time::SystemTime {
        self.now().into()
    }

    async fn sleep(&self, duration: Duration) {
        if duration.is_zero() {
            return;
        }

        let deadline = self.now() + duration;
        loop {
            if self.now() >= deadline {
                return;
            }

            self.notify.notified().await;
        }
    }
}

/// A deterministic entropy source backed by a seeded RNG.
///
/// Given the same seed, this will always produce the same sequence of
/// random values, enabling reproducible test runs.
pub struct DstEntropy {
    rng: Mutex<StdRng>,
    seed: u64,
}

impl DstEntropy {
    /// Create a new DST entropy source with the given seed.
    pub fn new(seed: u64) -> Self {
        Self {
            rng: Mutex::new(StdRng::seed_from_u64(seed)),
            seed,
        }
    }

    /// Get the seed used by this entropy source.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}

impl Entropy for DstEntropy {
    fn random_range(&self, range: std::ops::Range<u128>) -> u128 {
        self.rng.lock().random_range(range)
    }
}

/// A deterministic event source for controlling when and whether PostgreSQL
/// LISTEN/NOTIFY events are delivered to the worker.
///
/// This wraps a real [`PgEventSource`] (or any other `EventSource`) and
/// interposes a channel through which the test harness can:
///
/// - **Inject events** directly via [`send`](DstEventSource::send), without
///   waiting for PostgreSQL notifications.
/// - **Simulate connection loss** by sending [`Event::Lagged`].
/// - **Block event delivery** by not sending any events, pausing the worker's
///   event loop until the test is ready.
///
/// # Architecture
///
/// `DstEventSource` is split into two parts:
/// - [`DstEventSource`] implements [`EventSource`] and is given to the worker.
/// - [`DstEventHandle`] is retained by the test harness to inject events.
///
/// Create both with [`DstEventSource::new`].
///
/// [`PgEventSource`]: crate::worker::PgEventSource
pub struct DstEventSource {
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

/// Handle for injecting events into a [`DstEventSource`].
///
/// This is the test-side half of the event source. Use [`send`](Self::send) to
/// deliver events to the worker, or [`send_lagged`](Self::send_lagged) to
/// simulate a connection loss.
#[derive(Clone)]
pub struct DstEventHandle {
    tx: tokio::sync::mpsc::UnboundedSender<Event>,
}

impl DstEventSource {
    /// Create a new DST event source and its corresponding handle.
    ///
    /// The returned `DstEventSource` should be passed to
    /// [`WorkerBuilder::event_source`]. The `DstEventHandle` is retained by
    /// the test harness.
    pub fn new() -> (Self, DstEventHandle) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        (Self { rx }, DstEventHandle { tx })
    }
}

#[async_trait::async_trait]
impl EventSource for DstEventSource {
    async fn next(&mut self) -> anyhow::Result<Event> {
        match self.rx.recv().await {
            Some(event) => Ok(event),
            None => {
                // Channel closed — the test handle was dropped. Return Lagged
                // and then pend forever so the worker doesn't spin.
                std::future::pending().await
            }
        }
    }
}

impl DstEventHandle {
    /// Send an event to the worker.
    pub fn send(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    /// Simulate a connection loss by sending [`Event::Lagged`].
    pub fn send_lagged(&self) {
        let _ = self.tx.send(Event::Lagged);
    }

    /// Send a task event (new task or task became ready).
    pub fn send_task(&self, id: i64, running_on: Option<i64>) {
        self.send(Event::Task(crate::event::Task { id, running_on }));
    }

    /// Send a worker event (worker inserted or deleted).
    pub fn send_worker(&self, worker_id: i64) {
        self.send(Event::Worker(crate::event::Worker { worker_id }));
    }

    /// Send a notification event.
    pub fn send_notification(&self, task_id: i64, event: String) {
        self.send(Event::Notification(crate::event::Notification {
            task_id,
            event,
        }));
    }

    /// Send a task suspend event.
    pub fn send_task_suspend(&self, id: i64) {
        self.send(Event::TaskSuspend(crate::event::TaskSuspend { id }));
    }

    /// Check whether the worker-side receiver has been dropped.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn dst_entropy_is_deterministic() {
        let e1 = DstEntropy::new(42);
        let e2 = DstEntropy::new(42);

        let vals1: Vec<u128> = (0..100).map(|_| e1.random_range(0..1000)).collect();
        let vals2: Vec<u128> = (0..100).map(|_| e2.random_range(0..1000)).collect();
        assert_eq!(vals1, vals2);
    }

    #[test]
    fn dst_entropy_different_seeds_differ() {
        let e1 = DstEntropy::new(42);
        let e2 = DstEntropy::new(43);

        let vals1: Vec<u128> = (0..10).map(|_| e1.random_range(0..1000)).collect();
        let vals2: Vec<u128> = (0..10).map(|_| e2.random_range(0..1000)).collect();
        assert_ne!(vals1, vals2);
    }

    #[test]
    fn dst_clock_advance() {
        let clock = DstClock::new(Utc::now());
        let t0 = clock.now();

        clock.advance(Duration::from_secs(60));
        let t1 = clock.now();

        assert_eq!((t1 - t0).num_seconds(), 60);
    }

    #[test]
    fn dst_clock_set() {
        let t0 = DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let clock = DstClock::new(t0);

        let t1 = DateTime::parse_from_rfc3339("2025-06-15T12:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        clock.set(t1);

        assert_eq!(clock.now(), t1);
    }

    #[tokio::test]
    async fn dst_clock_sleep_completes_on_advance() {
        let clock = Arc::new(DstClock::new(Utc::now()));
        let clock2 = clock.clone();

        let handle = tokio::spawn(async move {
            clock2.sleep(Duration::from_secs(10)).await;
        });

        // Yield to let the sleep start
        tokio::task::yield_now().await;

        // Advance past the deadline
        clock.advance(Duration::from_secs(11));

        // The sleep should complete
        tokio::time::timeout(Duration::from_secs(1), handle)
            .await
            .expect("sleep should have completed after clock advance")
            .unwrap();
    }

    #[tokio::test]
    async fn dst_clock_sleep_zero_returns_immediately() {
        let clock = DstClock::new(Utc::now());
        clock.sleep(Duration::ZERO).await;
        // If this doesn't hang, the test passes
    }

    #[tokio::test]
    async fn dst_scheduler_records_acquires() {
        let scheduler = DstScheduler::new(42);

        let _guard = scheduler
            .acquire(Component::Heartbeat { worker_id: 1 })
            .await;
        let _guard = scheduler
            .acquire(Component::Leader { worker_id: 1 })
            .await;

        assert_eq!(scheduler.acquire_count(), 2);
        let acquires = scheduler.acquires();
        assert!(matches!(
            acquires[0],
            Component::Heartbeat { worker_id: 1 }
        ));
        assert!(matches!(acquires[1], Component::Leader { worker_id: 1 }));
    }

    #[tokio::test]
    async fn dst_scheduler_records_events() {
        let scheduler = DstScheduler::new(42);

        scheduler.notify(ScheduleEvent::WorkerRegistered { worker_id: 1 });
        scheduler.notify(ScheduleEvent::TaskClaimed {
            worker_id: 1,
            task_id: 10,
        });

        let events = scheduler.events();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            ScheduleEvent::WorkerRegistered { worker_id: 1 }
        ));
        assert!(matches!(
            events[1],
            ScheduleEvent::TaskClaimed {
                worker_id: 1,
                task_id: 10
            }
        ));
    }

    #[tokio::test]
    async fn dst_scheduler_clear() {
        let scheduler = DstScheduler::new(42);

        scheduler.notify(ScheduleEvent::WorkerRegistered { worker_id: 1 });
        let _guard = scheduler
            .acquire(Component::Heartbeat { worker_id: 1 })
            .await;

        assert_eq!(scheduler.events().len(), 1);
        assert_eq!(scheduler.acquire_count(), 1);

        scheduler.clear();

        assert_eq!(scheduler.events().len(), 0);
        assert_eq!(scheduler.acquires().len(), 0);
        assert_eq!(scheduler.acquire_count(), 0);
    }

    #[tokio::test]
    async fn dst_event_source_delivers_events() {
        let (mut source, handle) = DstEventSource::new();

        handle.send_task(42, None);
        handle.send_worker(1);
        handle.send_lagged();

        let e1 = source.next().await.unwrap();
        assert!(matches!(e1, Event::Task(ref t) if t.id == 42));

        let e2 = source.next().await.unwrap();
        assert!(matches!(e2, Event::Worker(ref w) if w.worker_id == 1));

        let e3 = source.next().await.unwrap();
        assert!(matches!(e3, Event::Lagged));
    }

    #[tokio::test]
    async fn dst_event_source_blocks_until_sent() {
        let (mut source, handle) = DstEventSource::new();

        // next() should block until we send something
        let recv = tokio::spawn(async move { source.next().await.unwrap() });

        // Yield to let recv start waiting
        tokio::task::yield_now().await;

        handle.send_notification(10, "wakeup".into());

        let event = tokio::time::timeout(Duration::from_secs(1), recv)
            .await
            .expect("should complete")
            .unwrap();

        assert!(matches!(event, Event::Notification(ref n) if n.task_id == 10));
    }

    #[tokio::test]
    async fn dst_event_handle_is_clone() {
        let (mut source, handle) = DstEventSource::new();
        let handle2 = handle.clone();

        handle.send_task(1, None);
        handle2.send_task(2, None);

        let e1 = source.next().await.unwrap();
        let e2 = source.next().await.unwrap();

        assert!(matches!(e1, Event::Task(ref t) if t.id == 1));
        assert!(matches!(e2, Event::Task(ref t) if t.id == 2));
    }

    #[tokio::test]
    async fn dst_scheduler_custom_component() {
        let scheduler = DstScheduler::new(42);

        let _guard = scheduler
            .acquire(Component::Custom("my-service".into()))
            .await;

        let acquires = scheduler.acquires();
        assert!(matches!(&acquires[0], Component::Custom(name) if name == "my-service"));
    }
}
