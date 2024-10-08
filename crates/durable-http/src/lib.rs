//! Make HTTP requests as part of your workflow.
//!
//! The easiest way to get started is to use the module-level methods to create
//! a [`RequestBuilder`].
//!
//! ```no_run
//! let response = durable::http::get("http://httpbin.org/ip")
//!     .send()
//!     .expect("failed to make an HTTP request");
//!
//! println!("{}", response.text().unwrap());
//! ```

use core::fmt;
use std::str::{FromStr, Utf8Error};
use std::string::FromUtf8Error;
use std::time::Duration;

use durable_core::transaction;
use http::header::{InvalidHeaderName, InvalidHeaderValue};
use serde::{Deserialize, Serialize};

mod bindings {
    #![allow(unused_braces, clippy::all)]

    include!("bindings.rs");

    pub use self::durable::core::http::*;
}

#[doc(inline)]
pub use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
#[doc(inline)]
pub use url::Url;

pub type Result<T, E = Error> = std::result::Result<T, E>;

fn send(request: &Request) -> Result<Response, Error> {
    use crate::bindings::*;

    let label = format!("durable::http::send({} {})", request.method, request.url);
    let result = crate::transaction::maybe_txn(&label, || {
        let original = &request;

        let headers: Vec<_> = request
            .headers
            .iter()
            .map(|(name, value)| HttpHeaderParam {
                name: name.as_str(),
                value: value.as_bytes(),
            })
            .collect();

        let req = HttpRequest2::new(request.method.as_str(), request.url.as_str())?;
        req.set_headers(&headers)?;

        if let Some(body) = &request.body {
            req.set_body(body);
        }

        if let Some(timeout) = request.timeout {
            let timeout = timeout.as_nanos().try_into().unwrap_or(u64::MAX);
            req.set_timeout(timeout);
        }

        match fetch2(req) {
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
                    url: original.url.clone(),
                })
            }
            Err(err) => Err(Error::from(err)),
        }
    });

    result.map_err(|e| e.with_url(request.url.clone()))
}

/// Create a [`RequestBuilder`] for a `GET` request.
pub fn get(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::get(url.as_ref())
}

/// Create a [`RequestBuilder`] for a `POST` request.
pub fn post(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::post(url.as_ref())
}

/// Create a [`RequestBuilder`] for a `PUT` request.
pub fn put(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::put(url.as_ref())
}

/// Create a [`RequestBuilder`] for a `PATCH` request.
pub fn patch(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::patch(url.as_ref())
}

/// Create a [`RequestBuilder`] for a `DELETE` request.
pub fn delete(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::delete(url.as_ref())
}

/// Create a [`RequestBuilder`] for a `HEAD` request.
pub fn head(url: impl AsRef<str>) -> RequestBuilder {
    RequestBuilder::head(url.as_ref())
}

/// A request which can be executed by calling [`send`].
///
/// [`send`]: Request::send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    #[serde(with = "http_serde_ext::method")]
    method: Method,
    url: Url,
    #[serde(with = "http_serde_ext::header_map")]
    headers: HeaderMap,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "bytes_or_string::option"
    )]
    body: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<Duration>,
}

impl Request {
    /// Construct a new request.
    pub fn new(method: Method, url: Url) -> Self {
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

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn url_mut(&mut self) -> &mut Url {
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
    url: Url,
    #[serde(with = "http_serde_ext::status_code")]
    status: StatusCode,
    #[serde(with = "http_serde_ext::header_map")]
    headers: HeaderMap,
    #[serde(with = "bytes_or_string")]
    body: Vec<u8>,
}

impl Response {
    /// Get the status code of this response.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Get the url that the request was made from.
    pub fn url(&self) -> &Url {
        &self.url
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

    /// Turn a response into an error if the server returned an error.
    pub fn error_for_status(self) -> Result<Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            Err(Error::from(ErrorKind::Status(status)).with_url(self.url))
        } else {
            Ok(self)
        }
    }
}

/// A builder to construct the properties of a [`Request`].
///
/// To construct a `RequestBuilder` you can either use [`new`] or one of the
/// methods named after HTTP verbs (e.g. [`RequestBuilder::post`]).
///
/// Once you've constructed your request you can get the [`Request`] by calling
/// [`build`]. Alternatively, you can send it immediately by calling [`send`].
///
/// [`new`]: RequestBuilder::new
/// [`build`]: RequestBuilder::build
/// [`send`]: RequestBuilder::send
pub struct RequestBuilder {
    request: Result<Request, Error>,
}

impl RequestBuilder {
    /// Construct a new `RequestBuilder` with the provided URL and method.
    pub fn new(method: Method, url: impl AsRef<str>) -> Self {
        Self::_new(method, url.as_ref())
    }

    fn _new(method: Method, url: &str) -> Self {
        let result = match Url::from_str(url) {
            Ok(uri) => Ok(Request::new(method, uri)),
            Err(e) => Err(ErrorKind::InvalidUri(e.to_string()).into()),
        };

        Self { request: result }
    }

    /// Construct a new `RequestBuilder` for a `GET` request.
    pub fn get(url: impl AsRef<str>) -> Self {
        Self::new(Method::GET, url)
    }

    /// Construct a new `RequestBuilder` for a `POST` request.
    pub fn post(url: impl AsRef<str>) -> Self {
        Self::new(Method::POST, url)
    }

    /// Construct a new `RequestBuilder` for a `PUT` request.
    pub fn put(url: impl AsRef<str>) -> Self {
        Self::new(Method::PUT, url)
    }

    /// Construct a new `RequestBuilder` for a `PATCH` request.
    pub fn patch(url: impl AsRef<str>) -> Self {
        Self::new(Method::PATCH, url)
    }

    /// Construct a new `RequestBuilder` for a `DELETE` request.
    pub fn delete(url: impl AsRef<str>) -> Self {
        Self::new(Method::DELETE, url)
    }

    /// Construct a new `RequestBuilder` for a `HEAD` request.
    pub fn head(url: impl AsRef<str>) -> Self {
        Self::new(Method::HEAD, url)
    }
}

impl RequestBuilder {
    /// Add a header to this request.
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

    /// Add a set of headers to the existing ones on this request.
    ///
    /// The headers will be merged in to any already set.
    pub fn headers(self, headers: HeaderMap) -> Self {
        self.modify(|req| {
            replace_headers(&mut req.headers, headers);

            Ok(())
        })
    }

    /// Enables a request timeout.
    ///
    /// The timeout is applied from when the request starts connecting until the
    /// response body has finished.
    ///
    /// Note that the durable runtime has a configurable maximum timeout.
    /// Requests without a timeout or ones whose timeout is larger than the
    /// allowed maximum will have their timeout clamped to the configured
    /// maximum timeout.
    pub fn timeout(self, timeout: Duration) -> Self {
        const MAX_TIMEOUT: Duration = Duration::from_nanos(u64::MAX);

        self.modify(|request| {
            request.timeout = Some(timeout.min(MAX_TIMEOUT));
            Ok(())
        })
    }

    /// Set the request body.
    pub fn body(self, body: Vec<u8>) -> Self {
        self.modify(|request| {
            request.body = Some(body);
            Ok(())
        })
    }

    /// Set the request body to a JSON value.
    ///
    /// This will also add a `Content-Type: application/json` header to the list
    /// of headers.
    ///
    /// # Errors
    /// Serialization can fail if `T`'s implementation of `Serialize` decides to
    /// fail, or `T` contains a map with non-string keys.
    pub fn json<T: Serialize + ?Sized>(self, body: &T) -> Self {
        const CONTENT_TYPE: HeaderName = HeaderName::from_static("content-type");
        const APPLICATION_JSON: HeaderValue = HeaderValue::from_static("application/json");

        self.modify(|req| {
            let body = serde_json::to_vec(body).map_err(|e| ErrorKind::Other(e.to_string()))?;
            req.body = Some(body);
            req.headers.insert(CONTENT_TYPE, APPLICATION_JSON);
            Ok(())
        })
    }

    /// Modify the query string of the URL.
    ///
    /// Modifies the URL of this request, adding the parameters provided. This
    /// method appends and does not overwrite the parameters. This means that it
    /// can be called multiple times and that existing query parameters are not
    /// overwritten if the same key is used. The key will simply show up twice
    /// in the query string.
    ///
    /// # Note
    /// This method does not support serializing a single key-value pair.
    /// Instead of using `.query(("key", "value"))`, use a sequence, such as
    /// `.query(&[("key", "value")])`. It is also possible to serialize structs
    /// and maps into a key-value pair.
    ///
    /// # Errors
    /// This method will fail if the object you provide cannot be serialized
    /// into a query string.
    pub fn query<T: Serialize + ?Sized>(self, query: &T) -> Self {
        self.modify(|req| {
            let mut pairs = req.url.query_pairs_mut();
            let ser = serde_urlencoded::Serializer::new(&mut pairs);

            query
                .serialize(ser)
                .map_err(|e| ErrorKind::Other(e.to_string()))?;
            Ok(())
        })
    }

    /// Send a form body.
    ///
    /// Sets the body to the url encoded serialization of hte passed value and
    /// also sets the `Content-Type: application/x-www-form-urlencoded` header.
    ///
    /// # Errors
    /// This method will fail if the object provided cannot be serialized into
    /// the url encoded format.
    pub fn form<T: Serialize + ?Sized>(self, form: &T) -> Self {
        const CONTENT_TYPE: HeaderName = HeaderName::from_static("content-type");
        const FORM_URLENCODED: HeaderValue =
            HeaderValue::from_static("application/x-www-form-urlencoded");

        self.modify(|req| {
            let body = serde_urlencoded::to_string(form) //
                .map_err(|e| ErrorKind::Other(e.to_string()))?;
            req.headers.insert(CONTENT_TYPE, FORM_URLENCODED);
            req.body = Some(body.into_bytes());

            Ok(())
        })
    }

    /// Build a [`Request`], which can be inspected, modified, and sent via
    /// [`Request::send`].
    pub fn build(self) -> Result<Request, Error> {
        self.request
    }

    /// Construct the request and send it to the target URL, returning the
    /// response.
    ///
    /// # Errors
    /// This method fails if there was an error while sending the request, or if
    /// an error was stored in the builder.
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

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.request {
            Ok(req) => f
                .debug_struct("RequestBuilder")
                .field("request", req)
                .finish(),
            Err(e) => f.debug_struct("RequestBuilder").field("error", e).finish(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Error(Box<ErrorImpl>);

impl Error {
    pub fn with_url(mut self, url: Url) -> Self {
        self.0.url = Some(url);
        self
    }

    pub fn without_url(mut self) -> Self {
        self.0.url = None;
        self
    }

    pub fn url(&self) -> Option<&Url> {
        self.0.url.as_ref()
    }

    pub fn is_builder(&self) -> bool {
        match &self.0.kind {
            ErrorKind::Bindings(err) => err.kind == BindingsErrorKind::Builder,
            ErrorKind::Status(_) => false,
            _ => true,
        }
    }

    pub fn is_timeout(&self) -> bool {
        matches!(&self.0.kind, ErrorKind::Bindings(err) if err.kind == BindingsErrorKind::Timeout)
    }

    pub fn is_request(&self) -> bool {
        matches!(&self.0.kind, ErrorKind::Bindings(err) if err.kind == BindingsErrorKind::Request)
    }

    pub fn is_connect(&self) -> bool {
        matches!(&self.0.kind, ErrorKind::Bindings(err) if err.kind == BindingsErrorKind::Connect)
    }

    pub fn is_status(&self) -> bool {
        matches!(self.0.kind, ErrorKind::Status(_))
    }

    pub fn status(&self) -> Option<StatusCode> {
        match self.0.kind {
            ErrorKind::Status(status) => Some(status),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorImpl {
    kind: ErrorKind,
    url: Option<Url>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ErrorKind {
    /// The underlying runtime emitted an error.
    Bindings(BindingsError),

    InvalidStatus(u16),
    InvalidUri(String),
    InvalidMethod,
    InvalidHeaderName,
    InvalidHeaderValue,
    InvalidUtf8 {
        valid_up_to: usize,
        error_len: Option<usize>,
    },
    Status(#[serde(with = "status_code")] StatusCode),
    Other(String),
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Self(Box::new(ErrorImpl { kind, url: None }))
    }
}

impl From<bindings::HttpError2> for Error {
    fn from(value: bindings::HttpError2) -> Self {
        ErrorKind::Bindings(value.into()).into()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.kind.fmt(f)?;

        if let Some(url) = self.url() {
            if !matches!(self.0.kind, ErrorKind::Bindings(_)) {
                write!(f, " for url ({url})")?;
            }
        }

        Ok(())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.0.kind)
            .field("url", &self.0.url)
            .finish()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bindings(err) => f.write_str(&err.message),
            Self::InvalidUri(msg) => write!(f, "invalid uri: {msg}"),
            Self::InvalidStatus(status) => write!(f, "{status} is not a valid HTTP status code"),
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
            Self::Status(ref status) => {
                let prefix = if status.is_client_error() {
                    "HTTP status client error"
                } else {
                    "HTTP status server error"
                };

                write!(f, "{prefix} ({status})")
            }
            Self::Other(e) => e.fmt(f),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
enum BindingsErrorKind {
    Timeout,
    Builder,
    Request,
    Connect,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BindingsError {
    kind: BindingsErrorKind,
    message: String,
}

impl From<bindings::HttpError2> for BindingsError {
    fn from(error: bindings::HttpError2) -> Self {
        let message = error.message();
        let kind = match &error {
            e if e.is_timeout() => BindingsErrorKind::Timeout,
            e if e.is_builder() => BindingsErrorKind::Builder,
            e if e.is_request() => BindingsErrorKind::Request,
            e if e.is_connect() => BindingsErrorKind::Connect,
            _ => BindingsErrorKind::Other,
        };

        Self { kind, message }
    }
}

impl fmt::Display for BindingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

mod bytes_or_string {
    use serde::{de, Deserializer, Serializer};

    pub fn serialize<S>(data: &[u8], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match std::str::from_utf8(data) {
            Ok(data) => ser.serialize_str(data),
            Err(_) => ser.serialize_bytes(data),
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a string or a byte sequence")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_bytes(v.as_bytes())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.to_vec())
            }
        }

        de.deserialize_any(Visitor)
    }

    pub mod option {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        #[derive(Serialize)]
        struct BorrowedBytes<'a>(#[serde(serialize_with = "super::serialize")] &'a [u8]);

        #[derive(Deserialize)]
        struct OwnedBytes(#[serde(serialize_with = "super::deserialize")] Vec<u8>);

        pub fn serialize<S>(data: &Option<Vec<u8>>, ser: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            data.as_ref().map(|data| BorrowedBytes(data)).serialize(ser)
        }

        pub fn deserialize<'de, D>(de: D) -> Result<Option<Vec<u8>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let data: Option<OwnedBytes> = Deserialize::deserialize(de)?;
            Ok(data.map(|bytes| bytes.0))
        }
    }
}

mod status_code {
    use http::StatusCode;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &StatusCode, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.as_u16().serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<StatusCode, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let status = u16::deserialize(de)?;
        let status = StatusCode::from_u16(status).map_err(Error::custom)?;

        Ok(status)
    }
}

fn replace_headers(dst: &mut HeaderMap, src: HeaderMap) {
    // This method is stolen from reqwest's util.rs.
    //
    // IntoIter of HeaderMap yields (Option<HeaderName>, HeaderValue). The first
    // time a nme is yielded it will be Some(name) and if there are more values with
    // the same name then the next yield will be None.

    use http::header::{Entry, OccupiedEntry};

    let mut prev_entry: Option<OccupiedEntry<_>> = None;

    for (key, value) in src {
        match key {
            Some(key) => match dst.entry(key) {
                Entry::Occupied(mut e) => {
                    e.insert(value);
                    prev_entry = Some(e);
                }
                Entry::Vacant(e) => {
                    let e = e.insert_entry(value);
                    prev_entry = Some(e);
                }
            },
            None => match prev_entry {
                Some(ref mut entry) => entry.append(value),
                None => unreachable!("HeaderMap::into_iter yielded None first"),
            },
        }
    }
}
