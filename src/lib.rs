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

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// The production API host. Override with `Client::new(key, base_url)` for
/// the EU region (`https://api.agentmail.eu`) or a mock server.
pub const DEFAULT_BASE_URL: &str = "https://api.agentmail.to";

/// The per-request timeout applied by [`Client::new`] (connect + response).
pub const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Everything that can go wrong talking to AgentMail.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// [`Client::from_env`] found no `AGENTMAIL_API_KEY`.
    #[error("AGENTMAIL_API_KEY is not set")]
    MissingApiKey,
    /// The request never completed: DNS, TLS, connect, or the
    /// [`DEFAULT_TIMEOUT`] elapsed.
    #[error("transport error: {0}")]
    Transport(#[from] reqwest::Error),
    /// A non-2xx answer from the API, with whatever body it sent.
    #[error("AgentMail answered {status}: {body}")]
    Api {
        /// The HTTP status the API answered with.
        status: reqwest::StatusCode,
        /// The response body, verbatim (AgentMail sends JSON error details).
        body: String,
    },
    /// A 2xx answer whose body didn't decode into the expected type - either
    /// a bug in this crate's wire shapes or a breaking change in the API.
    #[error("undecodable AgentMail response ({reason}): {body}")]
    Decode {
        /// Why deserialization failed.
        reason: String,
        /// The response body, verbatim.
        body: String,
    },
}

/// An authenticated handle on the AgentMail API. Cheap to clone-ish (it owns
/// a pooled `reqwest::Client`); construct once and share by reference.
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

// Manual impl so an accidental `{:?}` never prints the API key.
impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl Client {
    /// A client against `base_url` (see [`DEFAULT_BASE_URL`]), with a
    /// [`DEFAULT_TIMEOUT`] on every request.
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        // We build reqwest with rustls but no bundled crypto provider, so install
        // `ring` as the process default (no aws-lc-rs / cmake). This is a global,
        // set-once operation: it no-ops if the host application already installed
        // a provider, so it never overrides a deliberate choice.
        let _ = rustls::crypto::ring::default_provider().install_default();
        Client {
            http: reqwest::Client::builder()
                .timeout(DEFAULT_TIMEOUT)
                .build()
                // Infallible for these options; build() can only fail on
                // TLS-backend misconfiguration.
                .expect("reqwest client"),
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

    // ── Inboxes ──────────────────────────────────────────────────────────────

    /// POST /v0/inboxes, a new agent-owned email address. Free plans get
    /// `{username}@agentmail.to`; custom domains must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/inboxes",
            &[],
            Some(serde_json::to_value(inbox).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes (first page; see [`Client::list_inboxes_page`]).
    pub async fn list_inboxes(&self) -> Result<InboxList, Error> {
        self.list_inboxes_page(Page::default()).await
    }

    /// GET /v0/inboxes with pagination. Feed [`InboxList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_inboxes_page(&self, page: Page) -> Result<InboxList, Error> {
        self.request(reqwest::Method::GET, "/v0/inboxes", &page.query(), None)
            .await
    }

    /// GET /v0/inboxes/{inbox_id}
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            None,
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
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
            &[],
            Some(serde_json::to_value(message).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages (first page; see
    /// [`Client::list_messages_page`]).
    pub async fn list_messages(&self, inbox_id: &str) -> Result<MessageList, Error> {
        self.list_messages_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/messages with pagination. Feed
    /// [`MessageList::next_page_token`] back in as [`Page::page_token`]
    /// until it comes back `None`.
    pub async fn list_messages_page(
        &self,
        inbox_id: &str,
        page: Page,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &page.query(),
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
            &[],
            None,
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/{message_id}/reply -- reply to a
    /// message. The `to` field is derived from the parent message; only
    /// `text`/`html`/`cc`/`bcc` are caller-supplied.
    pub async fn reply_to_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/messages/{}/reply",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(serde_json::to_value(reply).expect("serializable")),
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/{message_id}/reply-all -- reply
    /// to all recipients of a message.
    pub async fn reply_all_to_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/messages/{}/reply-all",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(serde_json::to_value(reply).expect("serializable")),
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}/messages/{message_id} -- update a
    /// message's labels (read state is a label, e.g. add/remove `unread`).
    /// Returns the message id and its labels after the change, not the full
    /// message body.
    pub async fn update_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        update: UpdateMessage,
    ) -> Result<UpdatedMessage, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(serde_json::to_value(update).expect("serializable")),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}/messages/{message_id} -- permanently
    /// deletes a message.
    pub async fn delete_message(&self, inbox_id: &str, message_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
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
            &[],
            Some(serde_json::to_value(webhook).expect("serializable")),
        )
        .await
    }

    /// GET /v0/webhooks (first page; see [`Client::list_webhooks_page`]).
    pub async fn list_webhooks(&self) -> Result<WebhookList, Error> {
        self.list_webhooks_page(Page::default()).await
    }

    /// GET /v0/webhooks with pagination. Feed [`WebhookList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_webhooks_page(&self, page: Page) -> Result<WebhookList, Error> {
        self.request(reqwest::Method::GET, "/v0/webhooks", &page.query(), None)
            .await
    }

    /// DELETE /v0/webhooks/{webhook_id}
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/webhooks/{}", urlish(webhook_id)),
            &[],
            None,
        )
        .await
    }

    // ── Drafts ────────────────────────────────────────────────────────────────

    /// POST /v0/inboxes/{inbox_id}/drafts -- create a draft. Supply
    /// `in_reply_to` to create a reply draft (with `reply_all` to address
    /// the whole thread) or `forward_of` to create a forward draft.
    pub async fn create_draft(&self, inbox_id: &str, draft: CreateDraft) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/drafts", urlish(inbox_id)),
            &[],
            Some(serde_json::to_value(draft).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts (first page; see
    /// [`Client::list_drafts_page`]).
    pub async fn list_drafts(&self, inbox_id: &str) -> Result<DraftList, Error> {
        self.list_drafts_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts with pagination. Feed
    /// [`DraftList::next_page_token`] back in as [`Page::page_token`]
    /// until it comes back `None`.
    pub async fn list_drafts_page(&self, inbox_id: &str, page: Page) -> Result<DraftList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/drafts", urlish(inbox_id)),
            &page.query(),
            None,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts/{draft_id}
    pub async fn get_draft(&self, inbox_id: &str, draft_id: &str) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None,
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}/drafts/{draft_id} -- edit fields on an
    /// existing draft. Passing `None` for an `Option` field leaves it
    /// unchanged; the API expects the field to be omitted, not set to null,
    /// when unchanged.
    pub async fn update_draft(
        &self,
        inbox_id: &str,
        draft_id: &str,
        update: UpdateDraft,
    ) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            Some(serde_json::to_value(update).expect("serializable")),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}/drafts/{draft_id}
    pub async fn delete_draft(&self, inbox_id: &str, draft_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None,
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/drafts/{draft_id}/send -- send a draft.
    /// The draft is deleted after a successful send.
    pub async fn send_draft(&self, inbox_id: &str, draft_id: &str) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/drafts/{}/send",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None,
        )
        .await
    }

    // ── Drafts ─────────────────────────────────────────────────────────────────────

    /// GET /v0/inboxes/{inbox_id}/messages with filters and pagination.
    /// Pass [`MessageListFilters`] with any filter fields set to narrow
    /// results. Feed [`MessageList::next_page_token`] back in as
    /// [`MessageListFilters::page_token`] until it comes back `None`.
    pub async fn list_messages_filtered(
        &self,
        inbox_id: &str,
        filters: MessageListFilters,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &filters.query(),
            None,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/search with filters. Feed
    /// [`MessageList::next_page_token`] back in as
    /// [`MessageListFilters::page_token`] until it comes back `None`.
    pub async fn search_messages(&self, inbox_id: &str, query: &str) -> Result<MessageList, Error> {
        self.search_messages_page(inbox_id, query, MessageListFilters::default())
            .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/search with pagination.
    pub async fn search_messages_page(
        &self,
        inbox_id: &str,
        query: &str,
        filters: MessageListFilters,
    ) -> Result<MessageList, Error> {
        let mut q = filters.query();
        q.push(("q", query.to_string()));
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages/search", urlish(inbox_id)),
            &q,
            None,
        )
        .await
    }
}

/// Minimal percent-encoding for path segments (ids are URL-safe in practice;
/// this keeps a stray space, slash, or non-ASCII char from corrupting the
/// path). Encodes per UTF-8 byte, as percent-encoding requires.
fn urlish(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'@' | b'+' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

// ─── Types (wire shapes from the OpenAPI spec) ────────────────────────────────

/// Pagination controls for the `list_*_page` calls. `Default` is the API's
/// own defaults (first page, server-chosen page size).
#[derive(Clone, Debug, Default)]
pub struct Page {
    /// Maximum items per page (the API caps this server-side).
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
}

impl Page {
    fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .build()
    }
}

/// Request body for [`Client::create_inbox`]. All fields optional; the API
/// generates a username when none is given.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateInbox {
    /// Local part of the address; random when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// A verified domain (or subdomain of one); defaults to `agentmail.to`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Human-readable sender name shown in email clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Your own idempotency/reference id for this inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Arbitrary JSON stored alongside the inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// An agent-owned inbox, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Inbox {
    /// Unique id, used in every message call.
    pub inbox_id: String,
    /// The address itself, e.g. `my-agent@agentmail.to`.
    pub email: String,
    /// Human-readable sender name, when set.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Owning pod, when the account uses pods.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// Your reference id from creation, when set.
    #[serde(default)]
    pub client_id: Option<String>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One page of inboxes from [`Client::list_inboxes_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct InboxList {
    /// Total inboxes in the account (not just this page).
    pub count: u64,
    /// This page of inboxes.
    #[serde(default)]
    pub inboxes: Vec<Inbox>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Request body for [`Client::send_message`]. At least one recipient in `to`
/// and one of `text`/`html` are required by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SendMessage {
    /// Primary recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body (send both for multipart).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to attach to the sent message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// The API's acknowledgement of a send.
#[derive(Clone, Debug, Deserialize)]
pub struct SentMessage {
    /// Id of the message just sent.
    pub message_id: String,
    /// Thread the message was filed under.
    pub thread_id: String,
}

/// Request body for [`Client::reply_to_message`] and
/// [`Client::reply_all_to_message`]. The `to` field is derived from the
/// parent message; at least one of `text`/`html` is required by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ReplyToMessage {
    /// Carbon-copy recipients (in addition to those on the thread).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to attach to the reply.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// Request body for [`Client::update_message`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateMessage {
    /// Labels to add.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}

/// The API's response to [`Client::update_message`]: the message id and its
/// labels after the update.
#[derive(Clone, Debug, Deserialize)]
pub struct UpdatedMessage {
    /// Id of the updated message.
    pub message_id: String,
    /// The message's labels after the update.
    #[serde(default)]
    pub labels: Vec<String>,
}

/// A message as the API returns it. List items are a subset of the full
/// get-message shape; every non-id field defaults so both parse.
#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    /// Unique id within the inbox.
    pub message_id: String,
    /// Inbox the message belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Conversation thread id.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// Sender address.
    #[serde(default)]
    pub from: Option<String>,
    /// Recipient addresses.
    #[serde(default)]
    pub to: Vec<String>,
    /// Subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Short plain-text excerpt (list responses).
    #[serde(default)]
    pub preview: Option<String>,
    /// Full plain-text body (get responses).
    #[serde(default)]
    pub text: Option<String>,
    /// Full HTML body (get responses).
    #[serde(default)]
    pub html: Option<String>,
    /// Labels on the message (e.g. `received`, `unread`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// RFC 3339 send/receive timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// One page of messages from [`Client::list_messages_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct MessageList {
    /// Total messages in the inbox (not just this page).
    pub count: u64,
    /// This page of messages.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Request body for [`Client::create_webhook`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateWebhook {
    /// HTTPS endpoint to deliver events to.
    pub url: String,
    /// e.g. `["message.received"]`.
    pub event_types: Vec<String>,
    /// Limit delivery to these inboxes; empty means all.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inbox_ids: Vec<String>,
    /// Your own idempotency/reference id for this webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

/// A webhook subscription, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Webhook {
    /// Unique id, used to delete the subscription.
    pub webhook_id: String,
    /// The subscribed HTTPS endpoint.
    pub url: String,
    /// Signing secret, returned once, on creation.
    #[serde(default)]
    pub secret: Option<String>,
    /// Subscribed event types.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// Inboxes the subscription is limited to; empty means all.
    #[serde(default)]
    pub inbox_ids: Vec<String>,
    /// Whether the subscription currently delivers.
    pub enabled: bool,
}

/// One page of webhooks from [`Client::list_webhooks_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct WebhookList {
    /// Total webhooks in the account (not just this page).
    pub count: u64,
    /// This page of webhooks.
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Request body for [`Client::create_draft`]. At least one recipient or a
/// reply/forward-of reference and one of `text`/`html` are required by the
/// API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateDraft {
    /// Primary recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Message id this draft is in reply to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    /// When creating a reply draft, set true to reply to all recipients of
    /// the referenced message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_all: Option<bool>,
    /// Message id this draft is a forward of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_of: Option<String>,
    /// Labels to attach to the draft.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    /// Schedule send at this RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,
}

/// Request body for [`Client::update_draft`]. Every field is optional;
/// omitted fields are left unchanged on the server. Pass `Some(vec![])`
/// to clear a recipient field; pass `None` to leave it alone.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDraft {
    /// Primary recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to add.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_labels: Option<Vec<String>>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_labels: Option<Vec<String>>,
    /// Schedule send at this RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,
}

/// A draft message, as the API returns it. List items are a subset of the
/// full get-draft shape; every optional field defaults so both parse.
#[derive(Clone, Debug, Deserialize)]
pub struct Draft {
    /// Unique draft id.
    pub draft_id: String,
    /// Inbox the draft belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Primary recipients.
    #[serde(default)]
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(default)]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(default)]
    pub bcc: Vec<String>,
    /// Subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(default)]
    pub text: Option<String>,
    /// HTML body.
    #[serde(default)]
    pub html: Option<String>,
    /// Labels on the draft.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Message id this draft is replying to.
    #[serde(default)]
    pub in_reply_to: Option<String>,
    /// Whether the draft was created as a reply-all.
    #[serde(default)]
    pub reply_all: Option<bool>,
    /// Message id this draft is forwarding.
    #[serde(default)]
    pub forward_of: Option<String>,
    /// Scheduled send time (RFC 3339).
    #[serde(default)]
    pub send_at: Option<String>,
    /// Timestamp the draft was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Timestamp the draft was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Sender address (available in get responses).
    #[serde(default)]
    pub from: Option<String>,
    /// Attachments on the draft.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// One page of drafts from [`Client::list_drafts_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct DraftList {
    /// Total drafts in the inbox (not just this page).
    pub count: u64,
    /// This page of drafts.
    #[serde(default)]
    pub drafts: Vec<Draft>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Attachment metadata as returned in message, thread, and draft responses.
/// `download_url` (a short-lived presigned URL) is populated only by the
/// attachment-download endpoints; the other fields describe the attachment in
/// list and get responses.
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    /// Unique attachment id.
    pub attachment_id: String,
    /// File name.
    #[serde(default)]
    pub filename: Option<String>,
    /// Size in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// MIME content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// Content-Disposition (e.g. `inline` or `attachment`).
    #[serde(default)]
    pub content_disposition: Option<String>,
    /// Content-ID, for inline attachments referenced by the HTML body.
    #[serde(default)]
    pub content_id: Option<String>,
    /// Short-lived presigned URL to download the bytes; populated only by the
    /// attachment-download endpoints.
    #[serde(default)]
    pub download_url: Option<String>,
    /// When `download_url` expires (RFC 3339).
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Filters for [`Client::list_messages_filtered`] and
/// [`Client::search_messages_page`]. Pagination fields (`limit`,
/// `page_token`) live here because they share the same query-parameter
/// namespace as the filter fields.
#[derive(Clone, Debug, Default)]
pub struct MessageListFilters {
    /// Maximum items per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Filter by labels.
    pub labels: Vec<String>,
    /// Only messages before this timestamp (RFC 3339).
    pub before: Option<String>,
    /// Only messages after this timestamp (RFC 3339).
    pub after: Option<String>,
    /// Return oldest first instead of newest first.
    pub ascending: Option<bool>,
    /// Include spam messages.
    pub include_spam: Option<bool>,
    /// Include blocked messages.
    pub include_blocked: Option<bool>,
    /// Include unauthenticated messages.
    pub include_unauthenticated: Option<bool>,
    /// Include trashed messages.
    pub include_trash: Option<bool>,
    /// Filter by sender substring (repeatable).
    pub from: Vec<String>,
    /// Filter by recipient substring -- matches to, cc, or bcc (repeatable).
    pub to: Vec<String>,
    /// Filter by subject substring (repeatable).
    pub subject: Vec<String>,
}

impl MessageListFilters {
    fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .many("labels", &self.labels)
            .opt("before", self.before.as_ref())
            .opt("after", self.after.as_ref())
            .opt("ascending", self.ascending.as_ref())
            .opt("include_spam", self.include_spam.as_ref())
            .opt("include_blocked", self.include_blocked.as_ref())
            .opt(
                "include_unauthenticated",
                self.include_unauthenticated.as_ref(),
            )
            .opt("include_trash", self.include_trash.as_ref())
            .many("from", &self.from)
            .many("to", &self.to)
            .many("subject", &self.subject)
            .build()
    }
}

/// Accumulates query-string parameters, skipping absent ones. Shared by
/// [`Page`] and the per-resource filter builders so the list endpoints all
/// serialize their parameters the same way.
pub(crate) struct QueryBuilder(Vec<(&'static str, String)>);

impl QueryBuilder {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// Push `key=value` when `value` is `Some`; skip it otherwise.
    pub(crate) fn opt(mut self, key: &'static str, value: Option<&impl ToString>) -> Self {
        if let Some(value) = value {
            self.0.push((key, value.to_string()));
        }
        self
    }

    /// Push one `key=value` pair per element (repeated-key array parameters).
    pub(crate) fn many(mut self, key: &'static str, values: &[String]) -> Self {
        for value in values {
            self.0.push((key, value.clone()));
        }
        self
    }

    /// Finish and return the accumulated parameters.
    pub(crate) fn build(self) -> Vec<(&'static str, String)> {
        self.0
    }
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
        // Non-ASCII encodes per UTF-8 byte, not per code point.
        assert_eq!(urlish("café"), "caf%C3%A9");
        assert_eq!(urlish("😀"), "%F0%9F%98%80");
    }

    #[test]
    fn page_query_pairs() {
        assert!(Page::default().query().is_empty());
        let q = Page {
            limit: Some(10),
            page_token: Some("tok".into()),
        }
        .query();
        assert_eq!(
            q,
            vec![
                ("limit", "10".to_string()),
                ("page_token", "tok".to_string())
            ],
        );
    }
}
