//! The single HTTP chokepoint plus the per-resource method modules. Each
//! submodule adds an `impl Client` block for one API resource; they all funnel
//! through `request` here so auth, error mapping, and body handling live in
//! one place.

use crate::{Client, Error};
use reqwest::{Method, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;

mod attachments;
mod auth;
mod domains;
mod drafts;
mod inboxes;
mod messages;
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

    /// Send an authenticated request and return the raw (status, body) without
    /// decoding. All the shared request-building lives here so retries (added
    /// later) wrap exactly one place.
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
        let mut req = self
            .http
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.api_key);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(body);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Ok((status, text))
    }
}
