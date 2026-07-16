//! The single HTTP chokepoint plus the per-resource method modules. Each
//! submodule adds an `impl Client` block for one API resource; they all funnel
//! through `request` here so auth, error mapping, and body handling live in
//! one place.

use crate::{Client, Error};

mod drafts;
mod inboxes;
mod messages;
mod webhooks;

impl Client {
    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> Result<T, Error> {
        let mut req = self
            .http
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.api_key);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
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
}
