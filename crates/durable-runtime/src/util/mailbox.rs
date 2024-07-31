use std::pin::Pin;
use std::task::{Context, Poll};

use futures_util::{FutureExt, Stream};
use parking_lot::RwLock;
use tokio::sync::futures::Notified;
use tokio::sync::Notify;

#[derive(Debug)]
struct Data<T> {
    epoch: u64,
    value: T,
}

#[derive(Debug)]
pub(crate) struct Mailbox<T> {
    data: RwLock<Data<T>>,
    notify: Notify,
}

impl<T> Mailbox<T> {
    pub fn new(value: T) -> Self {
        Self {
            data: RwLock::new(Data { epoch: 0, value }),
            notify: Notify::new(),
        }
    }

    pub fn store(&self, value: T) {
        let mut data = self.data.write();
        let _old = std::mem::replace(&mut data.value, value);
        data.epoch += 1;

        // Ensure that _old gets dropped after we release the lock.
        drop(data);
    }
}

impl<T: Copy> Mailbox<T> {
    pub fn get(&self) -> T {
        let data = self.data.read();
        data.value
    }

    /// Get a stream of future values in the mailbox.
    ///
    /// Note that this stream does not include _all_ future values sent to the
    /// mailbox. The only guarantee is that if new values are sent to the stream
    /// then there will be a prompt wakeup.
    pub fn stream(&self) -> MailboxStream<T> {
        let data = self.data.read();

        MailboxStream {
            watermark: data.epoch,
            mailbox: self,
            notify: self.notify.notified(),
        }
    }
}

#[pin_project::pin_project]
pub(crate) struct MailboxStream<'a, T> {
    watermark: u64,
    mailbox: &'a Mailbox<T>,
    #[pin]
    notify: Notified<'a>,
}

impl<'a, T: Copy> MailboxStream<'a, T> {
    pub async fn next(mut self: Pin<&mut Self>) -> T {
        std::future::poll_fn(|cx| self.as_mut().poll_next(cx))
            .await
            .unwrap()
    }
}

impl<'a, T: Copy> Stream for MailboxStream<'a, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        if this.notify.as_mut().poll_unpin(cx).is_pending() {
            return Poll::Pending;
        }

        let data = this.mailbox.data.read();
        *this.watermark = data.epoch;
        let item = data.value;
        let notify = this.mailbox.notify.notified();
        drop(data);

        Pin::set(&mut this.notify, notify);
        Poll::Ready(Some(item))
    }
}
