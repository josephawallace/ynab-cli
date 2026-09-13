use std::{collections::VecDeque, fmt, sync::Arc, time::Duration};

use reqwest::{Method, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::sync::Mutex;
use url::Url;

use crate::models::{DataEnvelope, ErrorResponse, ValidationError};

const DEFAULT_BASE_URL: &str = "https://api.ynab.com/v1";
const MAX_REQUESTS_PER_HOUR: usize = 200;

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub attempts: u8,
    pub max_backoff: Duration,
}
impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            attempts: 3,
            max_backoff: Duration::from_secs(8),
        }
    }
}

#[derive(Debug, Error)]
pub enum ClientBuildError {
    #[error("rate limit must be between 1 and {MAX_REQUESTS_PER_HOUR} requests per hour")]
    InvalidRateLimit,
    #[error("HTTP client configuration failed: {0}")]
    Http(#[from] reqwest::Error),
}
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("transport failure: {0}")]
    Transport(#[source] reqwest::Error),
    #[error("request timed out")]
    Timeout,
    #[error("request serialization failed: {0}")]
    Serialization(#[source] serde_json::Error),
    #[error("request validation failed: {0}")]
    Validation(#[from] ValidationError),
    #[error("request URL cannot accept path segments")]
    InvalidBaseUrl,
    #[error("response body could not be decoded for HTTP {status}: {source}")]
    Decode {
        status: StatusCode,
        #[source]
        source: serde_json::Error,
    },
    #[error("API response {status}: {name}: {detail}")]
    Response {
        status: StatusCode,
        id: Option<String>,
        name: String,
        detail: String,
        retry_after: Option<Duration>,
    },
}
impl ApiError {
    #[must_use]
    pub const fn exit_is_response(&self) -> bool {
        matches!(self, Self::Response { .. })
    }
}

#[derive(Clone)]
pub struct ClientBuilder {
    token: SecretString,
    base_url: Url,
    timeout: Duration,
    user_agent: String,
    requests_per_hour: usize,
    retry: RetryPolicy,
}
impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("token", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("user_agent", &self.user_agent)
            .field("requests_per_hour", &self.requests_per_hour)
            .field("retry", &self.retry)
            .finish()
    }
}
impl ClientBuilder {
    #[must_use]
    pub fn new(token: SecretString) -> Self {
        Self {
            token,
            base_url: Url::parse(DEFAULT_BASE_URL).expect("static API URL"),
            timeout: Duration::from_secs(30),
            user_agent: format!("ynab-sdk/{}", env!("CARGO_PKG_VERSION")),
            requests_per_hour: 180,
            retry: RetryPolicy::default(),
        }
    }
    #[must_use]
    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = value;
        self
    }
    #[must_use]
    pub fn user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = value.into();
        self
    }
    pub fn requests_per_hour(mut self, value: usize) -> Result<Self, ClientBuildError> {
        if value == 0 || value > MAX_REQUESTS_PER_HOUR {
            return Err(ClientBuildError::InvalidRateLimit);
        }
        self.requests_per_hour = value;
        Ok(self)
    }
    #[must_use]
    pub fn retry_policy(mut self, value: RetryPolicy) -> Self {
        self.retry = value;
        self
    }
    /// Overrides the API origin for tests or a trusted proxy. The bearer token is sent to this URL.
    #[must_use]
    pub fn base_url(mut self, value: Url) -> Self {
        self.base_url = value;
        self
    }
    pub fn build(self) -> Result<Client, ClientBuildError> {
        let http = reqwest::Client::builder()
            .timeout(self.timeout)
            .user_agent(self.user_agent)
            .build()?;
        Ok(Client {
            inner: Arc::new(Inner {
                http,
                token: self.token,
                base_url: self.base_url,
                retry: self.retry,
                limiter: Mutex::new(RollingLimiter::new(self.requests_per_hour)),
            }),
        })
    }
}

struct RollingLimiter {
    limit: usize,
    entries: VecDeque<tokio::time::Instant>,
}
impl RollingLimiter {
    const WINDOW: Duration = Duration::from_secs(3600);
    fn new(limit: usize) -> Self {
        Self {
            limit,
            entries: VecDeque::new(),
        }
    }
    fn try_acquire(&mut self) -> Option<Duration> {
        let now = tokio::time::Instant::now();
        while self
            .entries
            .front()
            .is_some_and(|start| now.duration_since(*start) >= Self::WINDOW)
        {
            self.entries.pop_front();
        }
        if self.entries.len() < self.limit {
            self.entries.push_back(now);
            None
        } else {
            Some(Self::WINDOW.saturating_sub(
                now.duration_since(*self.entries.front().expect("full limiter has an entry")),
            ))
        }
    }
}
struct Inner {
    http: reqwest::Client,
    token: SecretString,
    base_url: Url,
    retry: RetryPolicy,
    limiter: Mutex<RollingLimiter>,
}
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}
impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("token", &"[REDACTED]")
            .field("base_url", &self.inner.base_url)
            .finish_non_exhaustive()
    }
}
impl Client {
    #[must_use]
    pub fn new(token: SecretString) -> Self {
        ClientBuilder::new(token)
            .build()
            .expect("fixed default configuration")
    }
    #[must_use]
    pub fn builder(token: SecretString) -> ClientBuilder {
        ClientBuilder::new(token)
    }
    pub(crate) async fn execute<
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
        B: Serialize + ?Sized,
    >(
        &self,
        method: Method,
        segments: &[&str],
        query: Option<&Q>,
        body: Option<&B>,
    ) -> Result<T, ApiError> {
        let body = body
            .map(serde_json::to_vec)
            .transpose()
            .map_err(ApiError::Serialization)?;
        let retries = if method == Method::GET {
            self.inner.retry.attempts.saturating_sub(1)
        } else {
            0
        };
        for attempt in 0..=retries {
            loop {
                let wait = { self.inner.limiter.lock().await.try_acquire() };
                match wait {
                    Some(delay) => tokio::time::sleep(delay).await,
                    None => break,
                }
            }
            let mut url = self.inner.base_url.clone();
            {
                let mut path = url
                    .path_segments_mut()
                    .map_err(|()| ApiError::InvalidBaseUrl)?;
                for segment in segments {
                    path.push(segment);
                }
            }
            let mut request = self
                .inner
                .http
                .request(method.clone(), url)
                .bearer_auth(self.inner.token.expose_secret());
            if let Some(query) = query {
                request = request.query(query);
            }
            if let Some(body) = &body {
                request = request
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(body.clone());
            }
            let response = match request.send().await {
                Ok(value) => value,
                Err(error) => {
                    if error.is_timeout() {
                        if attempt < retries {
                            self.backoff(attempt, None).await;
                            continue;
                        }
                        return Err(ApiError::Timeout);
                    }
                    if attempt < retries {
                        self.backoff(attempt, None).await;
                        continue;
                    }
                    return Err(classify_transport(error));
                }
            };
            let status = response.status();
            let retry_after =
                parse_retry_after(response.headers().get(reqwest::header::RETRY_AFTER));
            let bytes = match response.bytes().await {
                Ok(bytes) => bytes,
                Err(_) if attempt < retries => {
                    self.backoff(attempt, None).await;
                    continue;
                }
                Err(error) => return Err(classify_transport(error)),
            };
            if status.is_success() {
                return serde_json::from_slice::<DataEnvelope<T>>(&bytes)
                    .map(|envelope| envelope.data)
                    .map_err(|source| ApiError::Decode { status, source });
            }
            if attempt < retries
                && (status == StatusCode::TOO_MANY_REQUESTS
                    || status == StatusCode::SERVICE_UNAVAILABLE)
            {
                self.backoff(attempt, retry_after).await;
                continue;
            }
            let error = serde_json::from_slice::<ErrorResponse>(&bytes)
                .map_err(|source| ApiError::Decode { status, source })?;
            return Err(ApiError::Response {
                status,
                id: Some(error.error.id),
                name: error.error.name,
                detail: error.error.detail,
                retry_after,
            });
        }
        unreachable!("retry loop always returns")
    }
    async fn backoff(&self, attempt: u8, retry_after: Option<Duration>) {
        let delay = retry_after.unwrap_or_else(|| {
            let base = Duration::from_secs(1_u64 << u32::from(attempt.min(3)))
                .min(self.inner.retry.max_backoff);
            let jitter_limit = u64::try_from(base.as_millis() / 4).unwrap_or(u64::MAX);
            let jitter = Duration::from_millis(fastrand::u64(0..=jitter_limit));
            (base + jitter).min(self.inner.retry.max_backoff)
        });
        tokio::time::sleep(delay).await;
    }
}
fn classify_transport(error: reqwest::Error) -> ApiError {
    if error.is_timeout() {
        ApiError::Timeout
    } else {
        ApiError::Transport(error.without_url())
    }
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<Duration> {
    value
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
}
