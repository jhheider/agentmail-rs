use serde::Deserialize;

/// What an API key is scoped to.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScopeType {
    /// The whole organization.
    Organization,
    /// A single pod.
    Pod,
    /// A single inbox.
    Inbox,
    /// A scope type this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Who the current API key authenticates as, from `auth_me`.
#[derive(Clone, Debug, Deserialize)]
pub struct Identity {
    /// Whether the key is scoped to the organization, a pod, or an inbox.
    pub scope_type: ScopeType,
    /// Id of the scope (`organization_id`, `pod_id`, or `inbox_id`).
    pub scope_id: String,
    /// The organization the key belongs to.
    pub organization_id: String,
    /// The pod, when the key is pod-scoped.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// The inbox, when the key is inbox-scoped.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// The API key's own id.
    #[serde(default)]
    pub api_key_id: Option<String>,
}
