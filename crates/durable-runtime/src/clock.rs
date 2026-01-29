//! Clock trait for deterministic simulation testing (DST).
//!
//! The [`Clock`] trait controls the runtime's view of time for internal
//! scheduling decisions (heartbeats, leader wakeup checks, suspend margins).
//!
//! Note that WASM-visible time reads (wall clock, monotonic clock) are already
//! recorded by the transaction replay system. This trait covers the
//! runtime-internal time reads that affect component behavior.

use std::time::Duration;

use chrono::{DateTime, Utc};

/// Controls the runtime's view of time.
///
/// In production, the default [`SystemClock`] delegates to `chrono::Utc::now()`
/// and `tokio::time::sleep`. In DST, a custom clock can return controlled
/// values so that timer-based logic (heartbeat expiry, suspend wakeup,
/// cleanup age) is deterministic.
///
/// # Interaction with WASM time
///
/// The WASM-visible clock reads (`wasi:clocks/wall-clock.now` and
/// `wasi:clocks/monotonic-clock.now`) are wrapped in the transaction
/// system and their results are recorded in the event log. Those reads
/// also go through this trait, so a DST clock will make WASM time
/// consistent with runtime time.
#[async_trait::async_trait]
pub trait Clock: Send + Sync {
    /// Current UTC time.
    ///
    /// Replaces `chrono::Utc::now()` throughout the runtime.
    fn now(&self) -> DateTime<Utc>;

    /// Current system time.
    ///
    /// Replaces `std::time::SystemTime::now()` for WASI wall clock.
    fn system_time_now(&self) -> std::time::SystemTime {
        self.now().into()
    }

    /// Sleep for the given duration.
    ///
    /// In DST, this should complete when the simulated clock advances
    /// past the deadline rather than actually sleeping.
    async fn sleep(&self, duration: Duration);
}

/// The default clock using real system time.
pub struct SystemClock;

#[async_trait::async_trait]
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn system_time_now(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await
    }
}
