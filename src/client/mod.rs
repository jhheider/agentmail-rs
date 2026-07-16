//! The single HTTP chokepoint plus the per-resource method modules. Each
//! submodule adds an `impl Client` block for one API resource; they all funnel
//! through `request` here so auth, error mapping, and body handling live in
//! one place.

use crate::{Client, Error};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

mod agent;
mod api_keys;
mod attachments;
mod auth;
mod domains;
mod drafts;
mod inbox_events;
mod inboxes;
mod lists;
mod messages;
mod metrics;
mod organizations;
mod pods;
mod threads;
mod webhooks;

/// A body type for the bodiless request calls (`GET`/`DELETE`). Turbofish this
/// on `execute`/`request` when there is nothing to serialize.
type NoBody = ();

impl Client {
    /// Send an authenticated request and JSON-decode a 2xx body into `T`. Pass
    /// `Some(&value)` to serialize a JSON request body, or `None::<&NoBody>`
    /// for the bodiless verbs. This is the one HTTP chokepoint: every endpoint
    /// method funnels through here so auth, retries, and error mapping live in
    /// a single place.
    pub(crate) async fn request<T, B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
    ) -> Result<T, Error>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let (status, text) = self.execute(method, path, query, body).await?;
        if !status.is_success() {
            return Err(Error::Api { status, body: text });
        }
        // DELETE endpoints answer with an empty body; map that to null so
        // `()` (unit) deserializes.
        let text = if text.trim().is_empty() {
            "null"
        } else {
            &text
        };
        serde_json::from_str(text).map_err(|e| Error::Decode {
            reason: e.to_string(),
            body: text.to_string(),
        })
    }

    /// Send an authenticated bodiless request and return the raw response body
    /// as text, for the non-JSON endpoints (e.g. a domain zone file).
    pub(crate) async fn request_text(&self, method: Method, path: &str) -> Result<String, Error> {
        let (status, text) = self.execute(method, path, &[], None::<&NoBody>).await?;
        if !status.is_success() {
            return Err(Error::Api { status, body: text });
        }
        Ok(text)
    }

    /// Build an authenticated request. Shared by every attempt of `execute`.
    fn build_request<B>(
        &self,
        method: Method,
        url: &str,
        query: &[(&str, String)],
        body: Option<&B>,
    ) -> reqwest::RequestBuilder
    where
        B: Serialize + ?Sized,
    {
        let mut req = self.http.request(method, url).bearer_auth(&self.api_key);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        req
    }

    /// Send an authenticated request and return the raw (status, body) without
    /// decoding. With the `retries` feature, transient failures (timeout, 429,
    /// 5xx) are retried with backoff here, the single chokepoint every endpoint
    /// method funnels through.
    #[cfg(feature = "retries")]
    async fn execute<B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
    ) -> Result<(StatusCode, String), Error>
    where
        B: Serialize + ?Sized,
    {
        let url = format!("{}{path}", self.base_url);
        let policy = &self.retry_policy;
        let mut attempt: u32 = 0;
        loop {
            // Bodies are always in-memory, so a retry can re-serialize freely.
            match self
                .build_request(method.clone(), &url, query, body)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    let retry_after = parse_retry_after(resp.headers());
                    let text = resp.text().await.unwrap_or_default();
                    if is_retryable_status(status) && attempt < policy.max_retries {
                        let delay = retry_after.unwrap_or_else(|| policy.backoff(attempt));
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    return Ok((status, text));
                }
                Err(e) => {
                    if is_retryable_error(&e) && attempt < policy.max_retries {
                        tokio::time::sleep(policy.backoff(attempt)).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(Error::Transport(e));
                }
            }
        }
    }

    /// Single-attempt `execute`, used when the `retries` feature is off.
    #[cfg(not(feature = "retries"))]
    async fn execute<B>(
        &self,
        method: Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&B>,
    ) -> Result<(StatusCode, String), Error>
    where
        B: Serialize + ?Sized,
    {
        let url = format!("{}{path}", self.base_url);
        let resp = self.build_request(method, &url, query, body).send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Ok((status, text))
    }
}

#[cfg(feature = "retries")]
impl crate::RetryPolicy {
    /// Exponential backoff for `attempt` (0-based), capped at `max_delay`, with
    /// up to 25% added jitter derived from the wall clock (no `rand` dependency).
    fn backoff(&self, attempt: u32) -> std::time::Duration {
        let base = self.base_delay.as_millis() as u64;
        let cap = self.max_delay.as_millis() as u64;
        let grown = base.saturating_mul(1u64 << attempt.min(20)).min(cap).max(1);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64)
            .unwrap_or(0);
        let jitter = nanos % (grown / 4 + 1);
        std::time::Duration::from_millis(grown + jitter)
    }
}

/// Retry on request timeout, 429, and any 5xx.
#[cfg(feature = "retries")]
fn is_retryable_status(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

/// Retry transport errors that are transient (connect / timeout).
#[cfg(feature = "retries")]
fn is_retryable_error(e: &reqwest::Error) -> bool {
    e.is_timeout() || e.is_connect()
}

/// Parse an integer-seconds `Retry-After` header, capped at 30s. The HTTP-date
/// form is ignored (falls back to backoff).
#[cfg(feature = "retries")]
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<std::time::Duration> {
    let secs: u64 = headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()?;
    Some(std::time::Duration::from_secs(secs.min(30)))
}
