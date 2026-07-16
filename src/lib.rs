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

mod client;
mod types;
mod util;

#[cfg(feature = "webhook-verify")]
mod verify;

pub use types::*;
#[cfg(feature = "webhook-verify")]
pub use verify::*;

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
    /// [`Client::download_attachment`] was handed an attachment with no
    /// `download_url`. Only the attachment-download endpoints populate that
    /// field; fetch the attachment via `get_*_attachment` first.
    #[error("attachment has no download_url")]
    NoDownloadUrl,
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
}
