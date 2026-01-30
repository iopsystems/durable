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
        *now = *now + duration;
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
    async fn dst_scheduler_custom_component() {
        let scheduler = DstScheduler::new(42);

        let _guard = scheduler
            .acquire(Component::Custom("my-service".into()))
            .await;

        let acquires = scheduler.acquires();
        assert!(matches!(&acquires[0], Component::Custom(name) if name == "my-service"));
    }
}
