use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::iter::FusedIterator;
use std::ops::Deref;

/// A wrapper around a dynamic error type.
///
/// Types returned from durable transactions are required to be serializable and
/// deserializable. This is inconvenient when it comes to dealing with errors.
/// Many error types in Rust are not serializable. That is where this error
/// steps in: it can be serialized, even if the underlying error type cannot.
///
/// It does this by serializing all the messages involved in the error chain.
///
/// # Limitations
/// Ultimately, the way this error type works is by serializing the [`Display`]
/// representations of the errors involved. When it is deserialized the original
/// error types are no longer there. This occurs in cases where you might not
/// expect it to (e.g. transactions always serialize + deserialize their return
/// values to avoid inconsistencies when the workflow is restarted).
#[derive(Debug)]
pub struct Error(ErrorImpl);

impl Error {
    /// Create a new error object from any error type.
    ///
    /// The error type must be '`static`, but there are no other restrictions on
    /// it.
    pub fn new<E: StdError + 'static>(error: E) -> Self {
        error.into()
    }

    /// Create a new error object from a printable error message.
    ///
    /// If the argument implements [`std::error::Error`], prefer [`Error::new`]
    /// instead which preserves the underlying error's cause chain.
    pub fn msg<M: Display>(message: M) -> Self {
        Self::new(ErrorFrame {
            message: message.to_string(),
            source: None,
        })
    }

    /// An iterator over the chain of source errors contained by this error.
    ///
    /// This iterator will visit every error in the cause chain of this error
    /// object, beginning with the error that this error object was created
    /// from.
    pub fn chain(&self) -> Causes<'_> {
        Causes(self.source())
    }

    /// Convert this error into a type which implements [`std::error::Error`].
    ///
    /// This is useful for integration with other error libraries.
    pub fn into_std_error(self) -> impl StdError {
        self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Error {
    type Target = dyn StdError + 'static;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<dyn StdError> for Error {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        &**self
    }
}

impl<E> From<E> for Error
where
    E: StdError + 'static,
{
    fn from(error: E) -> Self {
        Self(ErrorImpl(Box::new(error)))
    }
}

struct ErrorImpl(Box<dyn StdError>);

impl fmt::Debug for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for ErrorImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl StdError for ErrorImpl {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.0)
    }
}

struct ErrorFrame {
    message: String,
    source: Option<Box<ErrorFrame>>,
}

impl Drop for ErrorFrame {
    fn drop(&mut self) {
        // Avoid stack overflows by implementing drop ourselves.
        let mut next = self.source.take();
        while let Some(mut current) = next {
            next = current.source.take();
        }
    }
}

impl fmt::Debug for ErrorFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("ErrorFrame");
        dbg.field("message", &self.message);
        if let Some(cause) = self.source.as_deref() {
            dbg.field("cause", cause);
        }
        dbg.finish()
    }
}

impl fmt::Display for ErrorFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl StdError for ErrorFrame {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|source| source as &dyn StdError)
    }
}

/// An iterator over
pub struct Causes<'a>(Option<&'a (dyn StdError + 'static)>);

impl<'a> Iterator for Causes<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.0.take()?;
        self.0 = current.source();
        Some(current)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.0 {
            Some(_) => (1, None),
            None => (0, Some(0)),
        }
    }
}

impl<'a> FusedIterator for Causes<'a> {}

mod serialization {
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    impl Serialize for Error {
        fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = ser.serialize_seq(None)?;

            for cause in self.chain() {
                // Avoid extra copies if we got an error frame
                if let Some(cause) = cause.downcast_ref::<ErrorFrame>() {
                    seq.serialize_element(&cause.message)?;
                } else {
                    seq.serialize_element(&cause.to_string())?;
                }
            }

            seq.end()
        }
    }

    impl<'de> Deserialize<'de> for Error {
        fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let messages = <Vec<String>>::deserialize(de)?;
            let frames = messages.into_iter().rev().fold(None, |acc, msg| {
                Some(Box::new(ErrorFrame {
                    message: msg,
                    source: acc,
                }))
            });

            match frames {
                Some(frames) => Ok(Self(ErrorImpl(frames))),
                None => Err(serde::de::Error::invalid_length(
                    0,
                    &"expected a sequence of non-zero length",
                )),
            }
        }
    }
}
