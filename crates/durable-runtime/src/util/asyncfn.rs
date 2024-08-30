use std::future::Future;

pub(crate) trait AsyncFnOnce<Args> {
    type Output;
    type Future: Future<Output = Self::Output>;

    fn call(self, args: Args) -> Self::Future;
}

macro_rules! declare_async_fn {
    ( $($as:ident)* ) => {
        impl<Fn, Fut, $($as,)*> AsyncFnOnce<( $( $as, )*)> for Fn
        where
            Fn: FnOnce($($as),*) -> Fut,
            Fut: Future
        {
            type Output = <Fut as Future>::Output;
            type Future = Fut;

            #[allow(non_snake_case)]
            fn call(self, ($($as,)*): ($($as,)*)) -> Self::Future {
                self($($as),*)
            }
        }
    }
}

declare_async_fn!();
declare_async_fn!(A0);
declare_async_fn!(A0 A1);
declare_async_fn!(A0 A1 A2);
