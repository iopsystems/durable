use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::panic::AssertUnwindSafe;

use serde::de::DeserializeOwned;
use serde::Serialize;

static IN_TRANSACTION: SyncUnsafeCell<bool> = SyncUnsafeCell::new(false);

/// Create an execute a transaction.
///
/// At its core, a transaction is a set of actions that are executed together.
/// They will be executed as a unit. If the workflow gets interrupted in the
/// middle of a transaction then the transaction will be retried from the start.
/// Once the transaction has completed, however, it will not be executed again.
/// This means that transactions provide at-least-once semantics.
pub fn transaction<F, T>(label: &str, func: F) -> T
where
    F: Fn() -> T,
    T: Serialize + DeserializeOwned,
{
    let opts = TransactionOptions {
        label,
        is_txn: false,
    };

    transaction_with(opts, func)
}

pub fn in_transaction() -> bool {
    // SAFETY: Workflows are only run in a single-threaded environment so this is
    //         safe.
    unsafe { std::ptr::read(IN_TRANSACTION.get()) }
}

/// Run `func` in a transaction unless we are already running in one.
pub fn maybe_txn<F, T>(label: &str, func: F) -> T
where
    F: Fn() -> T,
    T: Serialize + DeserializeOwned,
{
    if !in_transaction() {
        let opts = TransactionOptions {
            label,
            is_txn: false,
        };

        transaction_with(opts, func)
    } else {
        func()
    }
}

pub struct TransactionOptions<'a> {
    label: &'a str,
    is_txn: bool,
}

impl<'a> TransactionOptions<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            is_txn: false,
        }
    }

    pub fn database(mut self, is_db_txn: bool) -> Self {
        self.is_txn = is_db_txn;
        self
    }
}

struct InTxnGuard(());

impl InTxnGuard {
    pub fn new() -> Self {
        let in_txn = unsafe { std::ptr::replace(IN_TRANSACTION.get(), true) };
        if in_txn {
            panic!("attempted to start a transaction while aready within another");
        }

        Self(())
    }
}

impl Drop for InTxnGuard {
    fn drop(&mut self) {
        unsafe { std::ptr::write(IN_TRANSACTION.get(), false) };
    }
}

pub fn transaction_with<F, T>(opts: TransactionOptions, func: F) -> T
where
    F: Fn() -> T,
    T: Serialize + DeserializeOwned,
{
    #[derive(Serialize, Deserialize)]
    #[serde(tag = "type", content = "data", rename_all = "kebab-case")]
    enum TransactionResult<T, E = String> {
        Value(T),
        Panic(E),
    }

    // We need to replace panics with unknown panic payloads with a known panic
    // payload so that future retries get the exact same panic.
    const UNKNOWN_PANIC_MESSAGE: &str = "the transaction panicked with an unknown payload";

    if let Some(data) = crate::bindings::transaction_enter(opts.label, opts.is_txn) {
        let data: TransactionResult<T> = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(e) => unreachable!("saved task data was invalid json: {e}"),
        };

        match data {
            TransactionResult::Value(data) => return data,
            TransactionResult::Panic(payload) => {
                std::panic::resume_unwind(Box::new(payload));
            }
        }
    }

    let _guard = InTxnGuard::new();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let data = func();
        let json = serde_json::to_string(&TransactionResult::<_, String>::Value(&data))
            .expect("failed to serialize the transaction result to json");

        (data, json)
    }));

    match result {
        Ok((data, json)) => {
            crate::bindings::transaction_exit(&json);

            data
        }
        Err(payload) => {
            let message: Box<String> = match payload.downcast::<String>() {
                Ok(message) => message,
                Err(payload) => {
                    if let Some(&message) = payload.downcast_ref::<&str>() {
                        Box::new(message.to_owned())
                    } else {
                        Box::new(UNKNOWN_PANIC_MESSAGE.to_owned())
                    }
                }
            };

            let json = match serde_json::to_string(&TransactionResult::<(), _>::Panic(&message)) {
                Ok(json) => json,
                Err(e) => crate::abort(&format!("failed to serialize panic message to json: {e}")),
            };

            crate::bindings::transaction_exit(&json);
            std::panic::resume_unwind(message);
        }
    }
}

struct SyncUnsafeCell<T>(UnsafeCell<T>);

impl<T> SyncUnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }
}

unsafe impl<T> Send for SyncUnsafeCell<T> {}
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

impl<T> Deref for SyncUnsafeCell<T> {
    type Target = UnsafeCell<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for SyncUnsafeCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
