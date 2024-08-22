use std::time::Duration;

use http::{HeaderName, HeaderValue, Method};

use crate::bindings::durable::core::http::*;
use crate::Task;

impl Task {
    async fn fetch_impl(&mut self, request: HttpRequest) -> Result<HttpResponse, HttpError> {
        let config = self.state.config();
        let client = self.state.client();

        let method = Method::from_bytes(request.method.as_bytes())?;
        let timeout = request
            .timeout
            .map(Duration::from_nanos)
            .unwrap_or(config.max_http_timeout)
            .min(config.max_http_timeout);

        let url = reqwest::Url::parse(&request.url) //
            .map_err(|e| HttpError::InvalidUrl(e.to_string()))?;
        let mut builder = client.request(method, url).timeout(timeout);

        if let Some(body) = request.body {
            builder = builder.body(body);
        }

        for header in request.headers {
            let name = HeaderName::from_bytes(header.name.as_bytes())?;
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

#[async_trait::async_trait]
impl Host for Task {
    async fn fetch(
        &mut self,
        request: HttpRequest,
    ) -> anyhow::Result<Result<HttpResponse, HttpError>> {
        self.state
            .assert_in_transaction("durable:http/http.fetch")?;

        Ok(self.fetch_impl(request).await)
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
