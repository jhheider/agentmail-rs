use serde::{Deserialize, Serialize};

/// An API key's metadata (never its secret material; see [`CreatedApiKey`]).
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKey {
    /// Unique key id.
    pub api_key_id: String,
    /// The key's non-secret prefix, for identification.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,
    /// The pod the key is scoped to, when applicable.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// The inbox the key is scoped to, when applicable.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// When the key was last used (RFC 3339).
    #[serde(default)]
    pub used_at: Option<String>,
    /// The key's permissions.
    #[serde(default)]
    pub permissions: Option<serde_json::Value>,
    /// When the key was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Request body for [`Client::create_api_key`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateApiKey {
    /// Human-readable name for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The permissions to grant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<serde_json::Value>,
}

/// The response to [`Client::create_api_key`]. The full `api_key` secret is
/// returned exactly once, here; store it now, as it cannot be retrieved again.
#[derive(Clone, Debug, Deserialize)]
pub struct CreatedApiKey {
    /// Unique key id.
    pub api_key_id: String,
    /// The full secret key. Shown only on creation.
    pub api_key: String,
    /// The key's non-secret prefix.
    #[serde(default)]
    pub prefix: Option<String>,
    /// Human-readable name.
    #[serde(default)]
    pub name: Option<String>,
    /// When the key was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One page of API keys from [`Client::list_api_keys_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKeyList {
    /// Total keys in the account (not just this page).
    pub count: u64,
    /// This page of keys.
    #[serde(default)]
    pub api_keys: Vec<ApiKey>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
