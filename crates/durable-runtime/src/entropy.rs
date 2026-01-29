//! Entropy trait for deterministic simulation testing (DST).
//!
//! The [`Entropy`] trait controls randomness for runtime-internal decisions.
//! WASM-visible randomness is already recorded by the transaction replay
//! system. This trait covers the runtime infrastructure's own use of
//! randomness (currently just heartbeat jitter).

/// Controls randomness for runtime-internal decisions.
///
/// In production, the default [`SystemEntropy`] uses `rand::rng()`.
/// In DST, a custom implementation backed by a seeded RNG ensures
/// reproducible behavior.
pub trait Entropy: Send + Sync {
    /// Generate a random value in the given range `[low, high)`.
    ///
    /// The range is expressed as `u128` to accommodate any integer width.
    fn random_range(&self, range: std::ops::Range<u128>) -> u128;
}

/// The default entropy source using the system RNG.
pub struct SystemEntropy;

impl Entropy for SystemEntropy {
    fn random_range(&self, range: std::ops::Range<u128>) -> u128 {
        use rand::Rng;
        rand::rng().random_range(range)
    }
}
