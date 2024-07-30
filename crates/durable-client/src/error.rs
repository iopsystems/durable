pub struct DurableError(pub(crate) ErrorImpl);

#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DurableErrorKind {
    /// An error occurred while attempting to validate a WASM program.
    ///
    /// This can occur for a variety of reasons:
    /// * The provided bytes are not a valid WASM module.
    /// * The program uses features of WASM that are known not to be supported
    ///   by the durable runtime.
    /// * The program provided is not a WASM component.
    ///
    /// If you want to inspect the error more deeply then you can inspect the
    /// inner [`wasmparser::BinaryReaderError`]. Note that the case where the
    /// program is not a component is detected separately and so it won't have
    /// an inner error.
    ProgramValidation,

    /// An error occured when interacting with the database.
    ///
    /// The internal error here is [`sqlx::Error`].
    Database,
}

mod detail {
    // We name it like this so we can reuse the debug formatting impl.
    #[derive(Debug)]
    pub(crate) enum DurableError {
        ProgramValidation(wasmparser::BinaryReaderError),
        ProgramIsNotAComponent,
        Database(sqlx::Error),
        NonexistantWorkflowId(i64),
    }
}

use std::fmt;

pub(crate) use self::detail::DurableError as ErrorImpl;

impl fmt::Debug for DurableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for DurableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ErrorImpl::ProgramValidation(e) => write!(f, "program failed to validate: {e}"),
            ErrorImpl::ProgramIsNotAComponent => {
                write!(f, "expected a WASM component but got a WASM module instead")
            }
            ErrorImpl::Database(e) => e.fmt(f),
            ErrorImpl::NonexistantWorkflowId(id) => write!(f, "no workflow with id {id}"),
        }
    }
}

impl std::error::Error for DurableError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ErrorImpl::ProgramValidation(e) => Some(e),
            ErrorImpl::ProgramIsNotAComponent => None,
            ErrorImpl::Database(e) => Some(e),
            ErrorImpl::NonexistantWorkflowId(_) => None,
        }
    }
}

impl From<wasmparser::BinaryReaderError> for ErrorImpl {
    fn from(error: wasmparser::BinaryReaderError) -> Self {
        Self::ProgramValidation(error)
    }
}

impl From<sqlx::Error> for ErrorImpl {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<sqlx::Error> for DurableError {
    fn from(error: sqlx::Error) -> Self {
        Self(error.into())
    }
}

impl From<ErrorImpl> for DurableError {
    fn from(error: ErrorImpl) -> Self {
        Self(error)
    }
}
