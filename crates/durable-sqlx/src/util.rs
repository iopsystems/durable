//! Utilities for use elsewhere in the driver.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

use futures_core::Stream;

mod noop {
    use std::task::{RawWaker, RawWakerVTable};

    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, drop, drop, drop);

    unsafe fn clone(_: *const ()) -> RawWaker {
        waker()
    }

    pub(super) fn waker() -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
}

pub(crate) fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let mut future = std::pin::pin!(future);
    let waker = unsafe { Waker::from_raw(noop::waker()) };
    let mut cx = Context::from_waker(&waker);

    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            break v;
        }
    }
}

pub(crate) struct BlockingStream<S>(S);

impl<S> BlockingStream<S> {
    pub fn new(stream: S) -> Self {
        Self(stream)
    }
}

impl<S> Iterator for BlockingStream<S>
where
    S: Stream + Unpin,
{
    type Item = <S as Stream>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        Pin::new(self).next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<S> Iterator for Pin<&'_ mut BlockingStream<S>>
where
    S: Stream,
{
    type Item = <S as Stream>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        use futures_util::StreamExt;

        // SAFETY: The usual pin projection.
        let mut stream = unsafe { Pin::map_unchecked_mut(self.as_mut(), |this| &mut this.0) };

        block_on(stream.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
