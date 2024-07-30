use std::fmt;
use std::sync::{Mutex, PoisonError, TryLockError};

/// This type is meant to basically be a thread-safe [`Cell`].
///
/// It presents the same API as `Cell` but is thread safe and uses a mutex
/// internally. It is intended for very limited uses where we basically want to
/// be able to update a value but don't want to be holding onto the lock longer
/// than necessary.
///
/// [`Cell`]: std::cell::Cell
pub(crate) struct LockCell<T>(Mutex<T>);

impl<T> LockCell<T> {
    pub const fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }
}

impl<T> LockCell<T> {
    pub fn set(&self, value: T) {
        self.replace(value);
    }

    pub fn replace(&self, new: T) -> T {
        let mut guard = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        std::mem::replace(&mut guard, new)
    }
}

impl<T: Copy> LockCell<T> {
    pub fn get(&self) -> T {
        let guard = self.0.lock().unwrap_or_else(PoisonError::into_inner);

        *guard
    }
}

impl<T> fmt::Debug for LockCell<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let guard = match self.0.try_lock() {
            Ok(guard) => guard,
            Err(TryLockError::Poisoned(e)) => e.into_inner(),
            Err(TryLockError::WouldBlock) => return f.write_str("<locked>"),
        };

        <T as fmt::Debug>::fmt(&guard, f)
    }
}
