use std::time::Duration;

use http::{HeaderName, HeaderValue, Method};
use reqwest::Request;
use wasmtime::component::Resource;

use crate::bindings::durable::core::http::*;
use crate::{Config, Resourceable, Task};

impl Resourceable for HttpError2 {
    const NAME: &'static str = "http-error2";

    type Data = DurableHttpError;
}

impl Resourceable for HttpRequest2 {
    const NAME: &'static str = "http-request2";

    type Data = reqwest::Request;
}

impl Task {
    async fn fetch2_impl(&mut self, request: Request) -> Result<HttpResponse, DurableHttpError> {
        let client = self.state.client();
        let response = client.execute(request).await?;

        Ok(HttpResponse {
            status: response.status().as_u16(),
            headers: response
                .headers()
                .iter()
                .map(|(name, value)| HttpHeader {
                    name: name.as_str().to_owned(),
                    value: value.as_bytes().to_owned(),
                })
                .collect(),
            body: response.bytes().await?.to_vec(),
        })
    }
}

#[async_trait::async_trait]
impl HostHttpError2 for Task {
    async fn message(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<String> {
        let error = self.resources.get(res)?;
        Ok(error.to_string())
    }

    async fn is_timeout(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<bool> {
        let error = self.resources.get(res)?;
        Ok(error.is_timeout())
    }

    async fn is_builder(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<bool> {
        let error = self.resources.get(res)?;
        Ok(error.is_builder())
    }

    async fn is_request(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<bool> {
        let error = self.resources.get(res)?;
        Ok(error.is_request())
    }

    async fn is_connect(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<bool> {
        let error = self.resources.get(res)?;
        Ok(error.is_connect())
    }

    async fn drop(&mut self, res: Resource<HttpError2>) -> wasmtime::Result<()> {
        self.resources.remove(res)?;
        Ok(())
    }
}

impl HttpRequest2 {
    #[allow(clippy::new_ret_no_self)]
    fn new(method: String, url: String, config: &Config) -> Result<Request, DurableHttpError> {
        let method = Method::from_bytes(method.as_bytes())?;
        let url = url::Url::parse(&url)?;

        let mut request = Request::new(method, url);
        *request.timeout_mut() = Some(config.max_http_timeout);

        Ok(request)
    }

    fn set_method(request: &mut Request, method: &str) -> Result<(), DurableHttpError> {
        let method = Method::from_bytes(method.as_bytes())?;
        *request.method_mut() = method;
        Ok(())
    }

    fn set_url(request: &mut Request, url: &str) -> Result<(), DurableHttpError> {
        let url = url::Url::parse(url)?;
        *request.url_mut() = url;
        Ok(())
    }

    fn set_headers(request: &mut Request, headers: &[HttpHeader]) -> Result<(), DurableHttpError> {
        let mut map = http::HeaderMap::with_capacity(headers.len());

        for header in headers {
            let name = HeaderName::from_bytes(header.name.as_bytes())?;
            let value = HeaderValue::from_bytes(&header.value)?;

            map.insert(name, value);
        }

        *request.headers_mut() = map;
        Ok(())
    }

    fn set_timeout(request: &mut Request, timeout: Duration, config: &Config) {
        *request.timeout_mut() = Some(timeout.min(config.max_http_timeout));
    }

    fn set_body(request: &mut Request, body: Vec<u8>) {
        *request.body_mut() = Some(reqwest::Body::from(body));
    }
}

#[async_trait::async_trait]
impl HostHttpRequest2 for Task {
    async fn new(
        &mut self,
        method: String,
        url: String,
    ) -> wasmtime::Result<Result<Resource<HttpRequest2>, Resource<HttpError2>>> {
        let config = self.state.config();

        Ok(match HttpRequest2::new(method, url, config) {
            Ok(request) => Ok(self.resources.insert(request)?),
            Err(e) => Err(self.resources.insert(e)?),
        })
    }

    async fn set_method(
        &mut self,
        res: Resource<HttpRequest2>,
        method: String,
    ) -> wasmtime::Result<Result<(), Resource<HttpError2>>> {
        let request = self.resources.get_mut(res)?;

        Ok(match HttpRequest2::set_method(request, &method) {
            Ok(()) => Ok(()),
            Err(e) => Err(self.resources.insert(e)?),
        })
    }

    async fn set_url(
        &mut self,
        res: Resource<HttpRequest2>,
        url: String,
    ) -> wasmtime::Result<Result<(), Resource<HttpError2>>> {
        let request = self.resources.get_mut(res)?;

        Ok(match HttpRequest2::set_url(request, &url) {
            Ok(()) => Ok(()),
            Err(e) => Err(self.resources.insert(e)?),
        })
    }

    async fn set_headers(
        &mut self,
        res: Resource<HttpRequest2>,
        headers: Vec<HttpHeader>,
    ) -> wasmtime::Result<Result<(), Resource<HttpError2>>> {
        let request = self.resources.get_mut(res)?;

        Ok(match HttpRequest2::set_headers(request, &headers) {
            Ok(()) => Ok(()),
            Err(e) => Err(self.resources.insert(e)?),
        })
    }

    async fn set_timeout(
        &mut self,
        res: Resource<HttpRequest2>,
        timeout: u64,
    ) -> wasmtime::Result<()> {
        let request = self.resources.get_mut(res)?;
        let config = self.state.config();

        HttpRequest2::set_timeout(request, Duration::from_nanos(timeout), config);
        Ok(())
    }

    async fn set_body(
        &mut self,
        res: Resource<HttpRequest2>,
        body: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let request = self.resources.get_mut(res)?;

        HttpRequest2::set_body(request, body);
        Ok(())
    }

    async fn drop(&mut self, res: Resource<HttpRequest2>) -> wasmtime::Result<()> {
        self.resources.remove(res)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Host for Task {
    async fn fetch(
        &mut self,
        request: HttpRequest,
    ) -> anyhow::Result<Result<HttpResponse, HttpError>> {
        self.state
            .assert_in_transaction("durable:http/http.fetch")?;

        let config = self.state.config();
        let result = (|| -> Result<_, DurableHttpError> {
            let mut req = HttpRequest2::new(request.method, request.url, config)?;
            HttpRequest2::set_headers(&mut req, &request.headers)?;

            if let Some(timeout) = request.timeout {
                HttpRequest2::set_timeout(&mut req, Duration::from_nanos(timeout), config);
            }

            if let Some(body) = request.body {
                HttpRequest2::set_body(&mut req, body);
            }

            Ok(req)
        })();

        let request = match result {
            Ok(request) => request,
            Err(e) => return Ok(Err(e.into())),
        };

        Ok(self.fetch2_impl(request).await.map_err(From::from))
    }

    async fn fetch2(
        &mut self,
        request: Resource<HttpRequest2>,
    ) -> wasmtime::Result<Result<HttpResponse, Resource<HttpError2>>> {
        self.state
            .assert_in_transaction("durable:http/http.fetch2")?;

        let request = self.resources.remove(request)?;

        Ok(match self.fetch2_impl(request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(self.resources.insert(e)?),
        })
    }
}

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

pub enum DurableHttpError {
    InvalidMethod(http::method::InvalidMethod),
    InvalidUrl(url::ParseError),
    InvalidHeaderName(http::header::InvalidHeaderName),
    InvalidHeaderValue(http::header::InvalidHeaderValue),
    Reqwest(reqwest::Error),
}

impl DurableHttpError {
    fn is_timeout(&self) -> bool {
        match self {
            Self::Reqwest(e) => e.is_timeout(),
            _ => false,
        }
    }

    fn is_builder(&self) -> bool {
        match self {
            Self::Reqwest(e) => e.is_builder(),
            _ => true,
        }
    }

    fn is_request(&self) -> bool {
        match self {
            Self::Reqwest(e) => e.is_request(),
            _ => false,
        }
    }

    fn is_connect(&self) -> bool {
        match self {
            Self::Reqwest(e) => e.is_connect(),
            _ => false,
        }
    }
}

impl From<http::method::InvalidMethod> for DurableHttpError {
    fn from(value: http::method::InvalidMethod) -> Self {
        Self::InvalidMethod(value)
    }
}

impl From<http::header::InvalidHeaderName> for DurableHttpError {
    fn from(value: http::header::InvalidHeaderName) -> Self {
        Self::InvalidHeaderName(value)
    }
}

impl From<http::header::InvalidHeaderValue> for DurableHttpError {
    fn from(value: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValue(value)
    }
}

impl From<url::ParseError> for DurableHttpError {
    fn from(value: url::ParseError) -> Self {
        Self::InvalidUrl(value)
    }
}

impl From<reqwest::Error> for DurableHttpError {
    fn from(value: reqwest::Error) -> Self {
        Self::Reqwest(value)
    }
}

impl From<DurableHttpError> for HttpError {
    fn from(error: DurableHttpError) -> Self {
        match error {
            DurableHttpError::InvalidMethod(_) => HttpError::InvalidMethod,
            DurableHttpError::InvalidHeaderName(_) => HttpError::InvalidHeaderName,
            DurableHttpError::InvalidHeaderValue(_) => HttpError::InvalidHeaderValue,
            DurableHttpError::InvalidUrl(err) => HttpError::InvalidUrl(err.to_string()),
            DurableHttpError::Reqwest(err) if err.is_timeout() => HttpError::Timeout,
            DurableHttpError::Reqwest(err) => HttpError::Other(err.to_string()),
        }
    }
}

impl std::fmt::Display for DurableHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMethod(err) => err.fmt(f),
            Self::InvalidUrl(err) => err.fmt(f),
            Self::InvalidHeaderName(err) => err.fmt(f),
            Self::InvalidHeaderValue(err) => err.fmt(f),
            Self::Reqwest(err) => err.fmt(f),
        }
    }
}
