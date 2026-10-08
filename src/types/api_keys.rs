use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An API key's permission flags, keyed by permission name (e.g.
/// `message_send`, `calendar_event_create`). Modeled as a map so permission
/// additions don't require a client release; unknown names are preserved.
pub type ApiKeyPermissions = BTreeMap<String, bool>;

/// The public-key material bound to a `public_key` API key (JWK plus its
/// RFC 7638 thumbprint), as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct PublicKeyMaterial {
    /// The public JWK (`kty`, `crv`, `x`, `y`).
    #[serde(default)]
    pub jwk: Option<serde_json::Value>,
    /// Unpadded base64url RFC 7638 SHA-256 thumbprint of the JWK.
    #[serde(default)]
    pub fingerprint: Option<String>,
}

/// The agent that created a `public_key` API key.
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKeyCreator {
    /// The creating key's id.
    pub api_key_id: String,
}

/// An API key, as the API returns it. The wire shape is a oneOf over bearer
/// keys and `public_key` credentials; every field defaults so both parse.
/// The secret material itself is never returned here (see [`CreatedApiKey`]).
#[derive(Clone, Debug, Deserialize)]
pub struct ApiKey {
    /// `bearer` or `public_key`.
    #[serde(rename = "type", default)]
    pub key_type: Option<String>,
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
    /// Your reference id, for `public_key` credentials.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Public-key material, for `public_key` credentials.
    #[serde(default)]
    pub public_key: Option<PublicKeyMaterial>,
    /// Registration state of a `public_key` credential: `pending` or `active`.
    #[serde(default)]
    pub status: Option<String>,
    /// When the key was last used (RFC 3339).
    #[serde(default)]
    pub used_at: Option<String>,
    /// The key's permissions.
    #[serde(default)]
    pub permissions: Option<ApiKeyPermissions>,
    /// The key that created this one, for `public_key` credentials.
    #[serde(default)]
    pub created_by: Option<ApiKeyCreator>,
    /// When the key was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
    /// When the key was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// When the key stops working (RFC 3339), when it expires.
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Request body for `create_api_key`. The default (no `public_key`) mints a
/// bearer key; setting `public_key` registers a `public_key` credential
/// instead. The full secret of a bearer key is returned exactly once, in
/// [`CreatedApiKey::api_key`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateApiKey {
    /// Human-readable name for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The permissions to grant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<ApiKeyPermissions>,
    /// When the key should stop working (RFC 3339).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Your reference id, when registering a `public_key` credential.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// The public JWK (`kty`, `crv`, `x`, `y`); its presence selects the
    /// `public_key` credential flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<serde_json::Value>,
}

/// Body for `update_api_key`. Fields left `None` stay unchanged.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateApiKey {
    /// Replace the human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Replace the permission set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<ApiKeyPermissions>,
}

/// The response to `create_api_key`. The full `api_key` secret is
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
    /// The pod the key is scoped to, when applicable.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// The inbox the key is scoped to, when applicable.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// The granted permissions.
    #[serde(default)]
    pub permissions: Option<ApiKeyPermissions>,
    /// When the key was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One page of API keys from `list_api_keys_page`.
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
