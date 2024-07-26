/// A permission token that allows you to perform "impure" operations.
pub struct TxnContext(());

impl TxnContext {
    pub(crate) fn new() -> Self {
        Self(())
    }
}
