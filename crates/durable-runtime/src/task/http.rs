use std::time::Duration;

use async_trait::async_trait;
use http::{HeaderName, HeaderValue, Method};

use super::WorkflowState;
use crate::bindings::durable::http::*;

impl WorkflowState {
    async fn http_impl(&mut self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let method = Method::from_bytes(request.method.as_bytes())?;
        let timeout = request
            .timeout
            .map(Duration::from_nanos)
            .unwrap_or(self.shared.config.max_http_timeout)
            .min(self.shared.config.max_http_timeout);

        let url = reqwest::Url::parse(&request.url) //
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        let mut builder = self.shared.client.request(method, url).timeout(timeout);

        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        for header in request.headers {
            let name = HeaderName::from_bytes(&header.name.as_bytes())?;
            let value = HeaderValue::from_bytes(&header.value)?;

            builder = builder.header(name, value);
        }

        let response = builder.send().await?;

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

#[async_trait]
impl Host for WorkflowState {
    async fn http(
        &mut self,
        request: HttpRequest,
    ) -> anyhow::Result<Result<HttpResponse, HttpError>> {
        self.assert_in_transaction("http")?;

        Ok(self.http_impl(request).await)
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
