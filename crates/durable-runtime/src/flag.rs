use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use tokio::sync::futures::Notified;
use tokio::sync::Notify;

struct Shared {
    state: AtomicBool,
    notify: Notify,
}

/// A one-shot async flag.
#[derive(Clone)]
pub struct ShutdownFlag(Arc<Shared>);

impl ShutdownFlag {
    pub fn new() -> Self {
        Self(Arc::new(Shared {
            state: AtomicBool::new(false),
            notify: Notify::new(),
        }))
    }

    pub fn raise(&self) {
        self.0.state.store(true, Ordering::Release);
        self.0.notify.notify_waiters();
    }

    pub fn is_raised(&self) -> bool {
        self.0.state.load(Ordering::Acquire)
    }

    pub fn reset(&self) {
        self.0.state.store(false, Ordering::Release);
    }

    pub fn wait(&self) -> ShutdownFuture {
        // Early check since there is no reason to create a Notified if it is not
        // necessary.
        //
        // We still need to check after creating the notified to avoid the case where
        // raise is called between when we checked the flag and when we constructed the
        // Notified.
        if self.is_raised() {
            return ShutdownFuture(None);
        }

        let notified = self.0.notify.notified();

        if self.is_raised() {
            ShutdownFuture(None)
        } else {
            ShutdownFuture(Some(notified))
        }
    }
}

pub struct ShutdownFuture<'a>(Option<Notified<'a>>);

impl Future for ShutdownFuture<'_> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // SAFETY: project Pin<&mut Self> -> Option<Pin<&mut Notified>>
        let notified = unsafe {
            let this = Pin::get_unchecked_mut(self);
            this.0.as_mut().map(|v| Pin::new_unchecked(v))
        };

        match notified {
            Some(notified) => notified.poll(cx),
            None => Poll::Ready(()),
        }
    }
}

pub struct ShutdownGuard<'a>(&'a ShutdownFlag);

impl<'a> ShutdownGuard<'a> {
    pub fn new(flag: &'a ShutdownFlag) -> Self {
        Self(flag)
    }
}

impl<'a> Drop for ShutdownGuard<'a> {
    fn drop(&mut self) {
        if !self.0.is_raised() {
            tracing::warn!(
                "durable worker task shutting down without the shutdown flag being raised"
            );
        }

        self.0.raise();
    }
}
