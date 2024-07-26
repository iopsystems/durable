use std::fmt;

#[derive(Copy, Clone, Debug, Default)]
pub struct AbortError;

impl fmt::Display for AbortError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "this task is on longer scheduled on the current worker and has been aborted"
        )
    }
}

impl std::error::Error for AbortError {}
