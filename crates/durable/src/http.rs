//! Make HTTP requests as part of your workflow.
//!
//! This module contains the

use core::fmt;
use std::str::{FromStr, Utf8Error};
use std::string::FromUtf8Error;
use std::time::Duration;

use http::header::{InvalidHeaderName, InvalidHeaderValue};
use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri};

fn send(request: &Request) -> Result<Response, Error> {
    use crate::bindings::*;

    crate::transaction::maybe_txn("durable::http::send", || {
        let method = request.method.as_str().to_owned();
        let url = request.url.to_string();
        let headers = request
            .headers
            .iter()
            .map(|(name, value)| HttpHeader {
                name: name.as_str().to_owned(),
                value: value.as_bytes().to_owned(),
            })
            .collect();
        let timeout = request
            .timeout
            .map(|t| t.as_nanos().try_into().unwrap_or(u64::MAX));

        let request = HttpRequest {
            method,
            url,
            headers,
            body: request.body.clone(),
            timeout,
        };

        match crate::bindings::http(&request) {
            Ok(response) => {
                let status = StatusCode::from_u16(response.status)
                    .map_err(|_| ErrorKind::InvalidStatus(response.status))?;
                let mut headers = HeaderMap::with_capacity(response.headers.len());
                for header in response.headers {
                    let name = HeaderName::from_str(&header.name)
                        .map_err(|_| ErrorKind::InvalidHeaderName)?;
                    let value = HeaderValue::from_bytes(&header.value)
                        .map_err(|_| ErrorKind::InvalidHeaderValue)?;

                    headers.append(name, value);
                }

                Ok(Response {
                    status,
                    headers,
                    body: response.body,
                })
            }
            Err(err) => Err(Error(match err {
                HttpError::Timeout => ErrorKind::Timeout,
                HttpError::InvalidMethod => ErrorKind::InvalidMethod,
                HttpError::InvalidUrl(msg) => ErrorKind::InvalidUri(msg),
                HttpError::InvalidHeaderName => ErrorKind::InvalidHeaderName,
                HttpError::InvalidHeaderValue => ErrorKind::InvalidHeaderValue,
                HttpError::Other(msg) => ErrorKind::Other(msg),
            })),
        }
    })
}

pub fn get(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

pub fn post(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

pub fn put(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

pub fn patch(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

pub fn delete(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

pub fn head(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    #[serde(with = "http_serde_ext::method")]
    method: Method,
    #[serde(with = "http_serde_ext::uri")]
    url: Uri,
    #[serde(with = "http_serde_ext::header_map")]
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
}

impl Request {
    /// Construct a new request.
    pub fn new(method: Method, url: Uri) -> Self {
        Self {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Get the method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get a mutable reference to a method.
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    pub fn url(&self) -> &Uri {
        &self.url
    }

    pub fn url_mut(&mut self) -> &mut Uri {
        &mut self.url
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    pub fn body(&self) -> Option<&Vec<u8>> {
        self.body.as_ref()
    }

    pub fn body_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.body
    }

    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    pub fn timeout_mut(&mut self) -> &mut Option<Duration> {
        &mut self.timeout
    }

    pub fn send(&self) -> Result<Response, Error> {
        send(self)
    }
}

/// A response to a submitted [`Request`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    #[serde(with = "http_serde_ext::status_code")]
    status: StatusCode,
    #[serde(with = "http_serde_ext::header_map")]
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Response {
    /// Get the status code of this response.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the headers of this response.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the headers of this response.
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the full response body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Convert this response into an owned response body.
    pub fn into_body(self) -> Vec<u8> {
        self.body
    }

    /// Remove the response body from this response, leaving behind an empty
    /// one.
    pub fn take_body(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.body)
    }

    pub fn text(&self) -> Result<&str, Utf8Error> {
        std::str::from_utf8(self.body())
    }

    pub fn into_text(self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.into_body())
    }

    /// Attempt to deserialize the response body as JSON.
    ///
    /// # Errors
    /// This method fails whenever the response body is not in JSON format or it
    /// cannot be properly deserialized to the target type `T`.
    pub fn json<'de, T>(&'de self) -> serde_json::Result<T>
    where
        T: serde::Deserialize<'de>,
    {
        serde_json::from_slice(&self.body)
    }
}

pub struct RequestBuilder {
    request: Result<Request, Error>,
}

impl RequestBuilder {
    pub fn new(method: Method, url: impl AsRef<str>) -> Self {
        Self::_new(method, url.as_ref())
    }

    fn _new(method: Method, url: &str) -> Self {
        let result = match Uri::from_str(url) {
            Ok(uri) => Ok(Request::new(method, uri)),
            Err(e) => Err(Error(ErrorKind::InvalidUri(e.to_string()))),
        };

        Self { request: result }
    }

    pub fn get(url: impl AsRef<str>) -> Self {
        Self::new(Method::GET, url)
    }

    pub fn post(url: impl AsRef<str>) -> Self {
        Self::new(Method::POST, url)
    }

    pub fn put(url: impl AsRef<str>) -> Self {
        Self::new(Method::PUT, url)
    }

    pub fn patch(url: impl AsRef<str>) -> Self {
        Self::new(Method::PATCH, url)
    }

    pub fn delete(url: impl AsRef<str>) -> Self {
        Self::new(Method::DELETE, url)
    }

    pub fn head(url: impl AsRef<str>) -> Self {
        Self::new(Method::HEAD, url)
    }

    pub fn header<K, V>(self, name: K, value: V) -> Self
    where
        HeaderName: TryFrom<K, Error = InvalidHeaderName>,
        HeaderValue: TryFrom<V, Error = InvalidHeaderValue>,
    {
        self.modify(|request| {
            let name: HeaderName = name.try_into().map_err(|_| ErrorKind::InvalidHeaderName)?;
            let value: HeaderValue = value
                .try_into()
                .map_err(|_| ErrorKind::InvalidHeaderValue)?;

            request.headers.append(name, value);

            Ok(())
        })
    }

    pub fn timeout(self, timeout: Duration) -> Self {
        const MAX_TIMEOUT: Duration = Duration::from_nanos(u64::MAX);

        self.modify(|request| {
            request.timeout = Some(timeout.min(MAX_TIMEOUT));
            Ok(())
        })
    }

    pub fn body(self, body: Vec<u8>) -> Self {
        self.modify(|request| {
            request.body = Some(body);
            Ok(())
        })
    }

    pub fn build(self) -> Result<Request, Error> {
        self.request
    }

    pub fn send(self) -> Result<Response, Error> {
        self.build()?.send()
    }

    fn modify<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut Request) -> Result<(), Error>,
    {
        if let Ok(request) = &mut self.request {
            if let Err(e) = func(request) {
                self.request = Err(e);
            }
        }

        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error(ErrorKind);

#[derive(Debug, Serialize, Deserialize)]
enum ErrorKind {
    Timeout,
    InvalidStatus(u16),
    InvalidUri(String),
    InvalidMethod,
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidUtf8 {
        valid_up_to: usize,
        error_len: Option<usize>,
    },
    Other(String),
}

impl From<ErrorKind> for Error {
    fn from(value: ErrorKind) -> Self {
        Self(value)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "the request timed out"),
            Self::InvalidUri(msg) => write!(f, "invalid uri: {msg}"),
            Self::InvalidStatus(status) => write!(f, "invalid HTTP status code: {status}"),
            Self::InvalidMethod => write!(f, "invalid HTTP method"),
            Self::InvalidHeaderName => write!(f, "invalid HTTP header name"),
            Self::InvalidHeaderValue => write!(f, "invalid HTTP header value"),
            Self::InvalidUtf8 {
                valid_up_to,
                error_len,
            } => {
                if let Some(error_len) = error_len {
                    write!(
                        f,
                        "invalid utf-8 sequence of {error_len} bytes from index {valid_up_to}"
                    )
                } else {
                    write!(f, "incomplete utf-8 byte sequence from index {valid_up_to}")
                }
            }
            Self::Other(e) => e.fmt(f),
        }
    }
}
