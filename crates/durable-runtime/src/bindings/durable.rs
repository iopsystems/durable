// #![allow(unused_mut)]

wasmtime::component::bindgen!({
    path: "../durable/wit",
    world: "durable:core/core",
    tracing: true,
    trappable_imports: true,
    async: {
        except_imports: [
            "task-name",
            "task-data",
            "abort"
        ]
    }
});

pub use self::durable::core::*;

mod impls {
    use super::http::HttpError;

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
}
