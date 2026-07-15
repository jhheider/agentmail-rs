//! Unofficial typed Rust client for [AgentMail](https://agentmail.to), the
//! email API for agents (official SDKs exist for Python and TypeScript; this
//! fills the Rust gap).
//!
//! Wire shapes follow AgentMail's OpenAPI spec (`docs.agentmail.to/openapi.json`,
//! API v0). Coverage focuses on the transactional core: inboxes, messages
//! (send/list/get), and webhooks. Everything deserializes permissively -
//! unknown fields are ignored, optional fields default, so spec additions
//! don't break callers.
//!
//! ```no_run
//! # async fn demo() -> Result<(), agentmail::Error> {
//! let client = agentmail::Client::from_env()?; // AGENTMAIL_API_KEY
//! let inbox = client
//!     .create_inbox(agentmail::CreateInbox {
//!         username: Some("my-agent".into()),
//!         display_name: Some("My Agent".into()),
//!         ..Default::default()
//!     })
//!     .await?;
//! client
//!     .send_message(
//!         &inbox.inbox_id,
//!         agentmail::SendMessage {
//!             to: vec!["someone@example.com".into()],
//!             subject: Some("Hello".into()),
//!             text: Some("From an agent's own inbox.".into()),
//!             ..Default::default()
//!         },
//!     )
//!     .await?;
//! # Ok(()) }
//! ```

use serde::{Deserialize, Serialize};

/// The production API host. Override with `Client::new(key, base_url)` for
/// the EU region (`https://api.agentmail.eu`) or a mock server.
pub const DEFAULT_BASE_URL: &str = "https://api.agentmail.to";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("AGENTMAIL_API_KEY is not set")]
    MissingApiKey,
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// A non-2xx answer from the API, with whatever body it sent.
    #[error("AgentMail answered {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl Client {
    /// A client against `base_url` (see [`DEFAULT_BASE_URL`]).
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        // We build reqwest with rustls but no bundled crypto provider, so install
        // `ring` as the process default (no aws-lc-rs / cmake). This is a global,
        // set-once operation: it no-ops if the host application already installed
        // a provider, so it never overrides a deliberate choice.
        let _ = rustls::crypto::ring::default_provider().install_default();
        Client {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
        }
    }

    /// From `AGENTMAIL_API_KEY` (+ optional `AGENTMAIL_BASE_URL`).
    pub fn from_env() -> Result<Self, Error> {
        let key = std::env::var("AGENTMAIL_API_KEY").map_err(|_| Error::MissingApiKey)?;
        let base =
            std::env::var("AGENTMAIL_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Ok(Self::new(key, base))
    }

    async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T, Error> {
        let mut req = self
            .http
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.api_key);
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
        serde_json::from_str(text).map_err(|e| Error::Api {
            status,
            body: format!("undecodable body ({e}): {text}"),
        })
    }

    // ── Inboxes ──────────────────────────────────────────────────────────────

    /// POST /v0/inboxes, a new agent-owned email address. Free plans get
    /// `{username}@agentmail.to`; custom domains must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/inboxes",
            Some(serde_json::to_value(inbox).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes
    pub async fn list_inboxes(&self) -> Result<InboxList, Error> {
        self.request(reqwest::Method::GET, "/v0/inboxes", None)
            .await
    }

    /// GET /v0/inboxes/{inbox_id}
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            None,
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            None,
        )
        .await
    }

    // ── Messages ─────────────────────────────────────────────────────────────

    /// POST /v0/inboxes/{inbox_id}/messages/send
    pub async fn send_message(
        &self,
        inbox_id: &str,
        message: SendMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/messages/send", urlish(inbox_id)),
            Some(serde_json::to_value(message).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages
    pub async fn list_messages(&self, inbox_id: &str) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            None,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}
    pub async fn get_message(&self, inbox_id: &str, message_id: &str) -> Result<Message, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            None,
        )
        .await
    }

    // ── Webhooks ─────────────────────────────────────────────────────────────

    /// POST /v0/webhooks, subscribe an HTTPS endpoint to events
    /// (e.g. `message.received`). The response carries the signing `secret`
    /// exactly once; store it.
    pub async fn create_webhook(&self, webhook: CreateWebhook) -> Result<Webhook, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/webhooks",
            Some(serde_json::to_value(webhook).expect("serializable")),
        )
        .await
    }

    /// GET /v0/webhooks
    pub async fn list_webhooks(&self) -> Result<WebhookList, Error> {
        self.request(reqwest::Method::GET, "/v0/webhooks", None)
            .await
    }

    /// DELETE /v0/webhooks/{webhook_id}
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/webhooks/{}", urlish(webhook_id)),
            None,
        )
        .await
    }
}

/// Minimal percent-encoding for path segments (ids are URL-safe in practice;
/// this keeps a stray space or slash from corrupting the path).
fn urlish(segment: &str) -> String {
    segment
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '@' | '+' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

// ─── Types (wire shapes from the OpenAPI spec) ────────────────────────────────

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateInbox {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// A verified domain (or subdomain of one); defaults to `agentmail.to`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Inbox {
    pub inbox_id: String,
    /// The address itself, e.g. `my-agent@agentmail.to`.
    pub email: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub pod_id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InboxList {
    pub count: u64,
    #[serde(default)]
    pub inboxes: Vec<Inbox>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct SendMessage {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SentMessage {
    pub message_id: String,
    pub thread_id: String,
}

/// A message as the API returns it. List items are a subset of the full
/// get-message shape; every non-id field defaults so both parse.
#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    pub message_id: String,
    #[serde(default)]
    pub inbox_id: Option<String>,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Vec<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub preview: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub html: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageList {
    pub count: u64,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateWebhook {
    pub url: String,
    /// e.g. `["message.received"]`.
    pub event_types: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inbox_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Webhook {
    pub webhook_id: String,
    pub url: String,
    /// Signing secret, returned once, on creation.
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub event_types: Vec<String>,
    #[serde(default)]
    pub inbox_ids: Vec<String>,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WebhookList {
    pub count: u64,
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shapes_round_trip() {
        // Response example from the OpenAPI spec.
        let inbox: Inbox = serde_json::from_str(
            r#"{"pod_id":"pod_1","inbox_id":"ib_1","email":"x@agentmail.to",
                "display_name":"X","updated_at":"2024-01-15T09:30:00Z",
                "created_at":"2024-01-15T09:30:00Z","surprise_field":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.email, "x@agentmail.to");

        // List items are a subset of the get shape, both must parse.
        let list: MessageList = serde_json::from_str(
            r#"{"count":1,"messages":[{"message_id":"m1","thread_id":"t1",
                "from":"a@b.c","subject":"hi","preview":"…","timestamp":"2026-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        assert_eq!(list.messages[0].message_id, "m1");
        assert!(list.messages[0].text.is_none());

        // Requests omit empties so the API's validators stay quiet.
        let body = serde_json::to_value(SendMessage {
            to: vec!["a@b.c".into()],
            subject: Some("s".into()),
            text: Some("t".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"to":["a@b.c"],"subject":"s","text":"t"}),
        );
    }

    #[test]
    fn path_segments_stay_paths() {
        assert_eq!(urlish("ib_abc-123.x@y+z"), "ib_abc-123.x@y+z");
        assert_eq!(urlish("a/b c"), "a%2Fb%20c");
    }
}
