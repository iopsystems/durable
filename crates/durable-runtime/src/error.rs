use std::fmt;
use std::sync::Arc;

/// An error used to indicate various explicit exits when running within the
/// worker.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// The task is no longer scheduled on the current worker.
    ///
    /// We should not be making any changes to the database for this task.
    NotScheduledOnWorker,

    /// The task exited with an error.
    ExitFailure,

    /// The task exited successfully.
    ExitSuccess,

    /// The task has suspended itself.
    Suspend,
}

impl TaskStatus {
    pub(crate) fn is_final(self) -> bool {
        !matches!(self, Self::NotScheduledOnWorker | Self::Suspend)
    }
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotScheduledOnWorker => {
                write!(f, "this task is no longer scheduled on the current worker")
            }
            Self::ExitFailure => write!(f, "this task exited with an error"),
            Self::ExitSuccess => write!(f, "this task exited successfully"),
            Self::Suspend => write!(f, "this task has suspended itself"),
        }
    }
}

impl std::error::Error for TaskStatus {}

#[derive(Clone)]
pub(crate) struct ClonableAnyhowError(Arc<anyhow::Error>);

impl ClonableAnyhowError {
    pub fn new(err: anyhow::Error) -> Self {
        Self(Arc::new(err))
    }
}

impl fmt::Debug for ClonableAnyhowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for ClonableAnyhowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ClonableAnyhowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<anyhow::Error> for ClonableAnyhowError {
    fn from(error: anyhow::Error) -> Self {
        Self::new(error)
    }
}
