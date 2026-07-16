use serde::{Deserialize, Serialize};

/// A sending domain, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Domain {
    /// Unique domain id.
    pub domain_id: String,
    /// The domain name, e.g. `mail.example.com`.
    #[serde(default)]
    pub domain: Option<String>,
    /// Owning pod, when the account uses pods.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// Verification status (e.g. `pending`, `verified`).
    #[serde(default)]
    pub status: Option<String>,
    /// Whether bounce/complaint feedback is enabled.
    #[serde(default)]
    pub feedback_enabled: Option<bool>,
    /// Whether subdomains of this domain may be used for inboxes.
    #[serde(default)]
    pub subdomains_enabled: Option<bool>,
    /// The DNS records to add for verification and sending.
    #[serde(default)]
    pub records: Vec<VerificationRecord>,
    /// Your reference id from creation, when set.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Timestamp the domain was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Timestamp the domain was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A single DNS record to publish for a domain.
#[derive(Clone, Debug, Deserialize)]
pub struct VerificationRecord {
    /// DNS record type, e.g. `TXT`, `MX`, `CNAME`.
    #[serde(rename = "type")]
    pub record_type: String,
    /// The record name (host).
    pub name: String,
    /// The record value.
    pub value: String,
    /// Verification status of this record.
    #[serde(default)]
    pub status: Option<String>,
    /// Priority, for records that need one (e.g. `MX`).
    #[serde(default)]
    pub priority: Option<i64>,
}

/// Request body for [`Client::create_domain`](crate::Client::create_domain).
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateDomain {
    /// The domain name to add.
    pub domain: String,
    /// Enable bounce/complaint feedback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_enabled: Option<bool>,
    /// Allow subdomains of this domain to be used for inboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomains_enabled: Option<bool>,
}

/// Request body for [`Client::update_domain`](crate::Client::update_domain). Fields left `None` stay unchanged.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDomain {
    /// Enable or disable bounce/complaint feedback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_enabled: Option<bool>,
    /// Enable or disable subdomains for inbox use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomains_enabled: Option<bool>,
}

/// One page of domains from [`Client::list_domains_page`](crate::Client::list_domains_page).
#[derive(Clone, Debug, Deserialize)]
pub struct DomainList {
    /// Total domains in the account (not just this page).
    pub count: u64,
    /// This page of domains.
    #[serde(default)]
    pub domains: Vec<Domain>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
