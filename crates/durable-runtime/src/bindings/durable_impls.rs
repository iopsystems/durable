//! External impls on types in the bindings that are not autogenerated.

use super::durable::*;

impl From<reqwest::Error> for HttpError {
    fn from(value: reqwest::Error) -> Self {
        if value.is_timeout() {
            return HttpError::Timeout;
        }

        HttpError::Other(value.to_string())
    }
}

impl From<http::method::InvalidMethod> for HttpError {
    fn from(_: http::method::InvalidMethod) -> Self {
        Self::InvalidMethod
    }
}

impl From<http::header::InvalidHeaderName> for HttpError {
    fn from(_: http::header::InvalidHeaderName) -> Self {
        Self::InvalidHeaderName
    }
}

impl From<http::header::InvalidHeaderValue> for HttpError {
    fn from(_: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue
    }
}
