use serde::Deserialize;

/// The organization the current API key belongs to, from
/// `get_organization`.
#[derive(Clone, Debug, Deserialize)]
pub struct Organization {
    /// Unique organization id.
    pub organization_id: String,
    /// Current number of inboxes.
    #[serde(default)]
    pub inbox_count: Option<i64>,
    /// Current number of domains.
    #[serde(default)]
    pub domain_count: Option<i64>,
    /// Maximum inboxes allowed on the plan.
    #[serde(default)]
    pub inbox_limit: Option<i64>,
    /// Maximum domains allowed on the plan.
    #[serde(default)]
    pub domain_limit: Option<i64>,
    /// Billing plan type, when set.
    #[serde(default)]
    pub billing_type: Option<String>,
    /// Authentication provider type, when set.
    #[serde(default)]
    pub authentication_type: Option<String>,
    /// When the organization was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// When the organization was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}
