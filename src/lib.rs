//! Unofficial typed Rust client for [AgentMail](https://agentmail.to), the
//! email API for agents (official SDKs exist for Python and TypeScript; this
//! fills the Rust gap).
//!
//! Wire shapes follow AgentMail's OpenAPI spec (`docs.agentmail.to/openapi.json`,
//! API v0), with full coverage of the surface the official SDKs expose:
//! inboxes, threads, messages, drafts, attachments, webhooks, domains, pods,
//! allow/block lists, metrics, API keys, and agent onboarding. Everything
//! deserializes permissively: unknown fields are ignored, optional fields
//! default, so spec additions don't break callers.
//!
//! Resources that exist at more than one scope (threads, webhooks, lists,
//! domains, ...) are reached through a scope handle ([`Client::org`],
//! [`Client::inbox`], or [`Client::pod`]), so the compiler rejects an
//! operation the scope doesn't support. Inbox-only resources (messages, drafts)
//! live on [`Client::inbox`].
//!
//! ```no_run
//! # async fn demo() -> Result<(), agentmail::Error> {
//! let client = agentmail::Client::from_env()?; // AGENTMAIL_API_KEY
//! let inbox = client
//!     .org()
//!     .create_inbox(agentmail::CreateInbox {
//!         username: Some("my-agent".into()),
//!         display_name: Some("My Agent".into()),
//!         ..Default::default()
//!     })
//!     .await?; // my-agent@agentmail.to
//!
//! client
//!     .inbox(&inbox.inbox_id)
//!     .send_text("someone@example.com", "Hello", "From an agent's own inbox.")
//!     .await?;
//! # Ok(()) }
//! ```
//!
//! # Coverage
//!
//! Inboxes, threads, messages (send/reply/forward/raw/batch), drafts,
//! attachments, webhooks, domains, pods, allow/block lists, metrics, inbox
//! events, API keys, organization, auth, and agent onboarding. Every list call
//! is paginated (`Page`), and requests carry automatic retries with backoff.
//!
//! # Features
//!
//! - **`retries`** (default): automatic retries with exponential backoff. Turn
//!   it off with `default-features = false` to drop the direct `tokio`
//!   dependency; tune it with [`Client::with_retry_policy`].
//! - **`webhook-verify`** (off by default): `verify_webhook_signature` for
//!   Svix-signed webhook deliveries. Adds `ring` (already the rustls provider)
//!   and `base64`.

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod client;
mod types;
mod util;

#[cfg(feature = "webhook-verify")]
mod verify;

pub use types::*;
#[cfg(feature = "webhook-verify")]
#[cfg_attr(docsrs, doc(cfg(feature = "webhook-verify")))]
pub use verify::*;

pub use client::scope::{
    ApiKeys, Domains, Drafts, InboxScope, Inboxes, Lists, Metrics, OrgScope, PodScope, Scope,
    Scoped, Threads, Webhooks,
};

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
    /// A 2xx answer whose body didn't decode into the expected type: either
    /// a bug in this crate's wire shapes or a breaking change in the API.
    #[error("undecodable AgentMail response ({reason}): {body}")]
    Decode {
        /// Why deserialization failed.
        reason: String,
        /// The response body, verbatim.
        body: String,
    },
    /// [`Client::download_attachment`] was handed an attachment with no
    /// `download_url`. Only the attachment-download endpoints populate that
    /// field; fetch the attachment via `get_*_attachment` first.
    #[error("attachment has no download_url")]
    NoDownloadUrl,
}

/// How the client retries transient failures. Applied to every request at the
/// one HTTP chokepoint. `Default` retries twice with exponential backoff.
///
/// Requires the `retries` feature (on by default).
#[cfg(feature = "retries")]
#[cfg_attr(docsrs, doc(cfg(feature = "retries")))]
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    /// Extra attempts after the first. `0` disables retries.
    pub max_retries: u32,
    /// Base backoff delay; doubles each attempt.
    pub base_delay: std::time::Duration,
    /// Upper bound on a single backoff delay (before jitter).
    pub max_delay: std::time::Duration,
}

#[cfg(feature = "retries")]
impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_retries: 2,
            base_delay: std::time::Duration::from_millis(500),
            max_delay: std::time::Duration::from_secs(8),
        }
    }
}

/// An authenticated handle on the AgentMail API. Cheap to clone-ish (it owns
/// a pooled `reqwest::Client`); construct once and share by reference.
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    #[cfg(feature = "retries")]
    retry_policy: RetryPolicy,
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
            #[cfg(feature = "retries")]
            retry_policy: RetryPolicy::default(),
        }
    }

    /// From `AGENTMAIL_API_KEY` (+ optional `AGENTMAIL_BASE_URL`).
    pub fn from_env() -> Result<Self, Error> {
        let key = std::env::var("AGENTMAIL_API_KEY").map_err(|_| Error::MissingApiKey)?;
        let base =
            std::env::var("AGENTMAIL_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        Ok(Self::new(key, base))
    }

    /// Replace the [`RetryPolicy`]. Set `max_retries` to `0` to disable retries.
    ///
    /// Requires the `retries` feature (on by default).
    #[cfg(feature = "retries")]
    #[cfg_attr(docsrs, doc(cfg(feature = "retries")))]
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }
}
