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
    /// Why the domain is in its current state, when the API reports one.
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether bounce/complaint feedback is enabled.
    #[serde(default)]
    pub feedback_enabled: Option<bool>,
    /// Whether inbound mail to this domain is delivered to agent inboxes.
    #[serde(default)]
    pub inbound_enabled: Option<bool>,
    /// Whether subdomains of this domain may be used for inboxes.
    #[serde(default)]
    pub subdomains_enabled: Option<bool>,
    /// Whether open/click tracking is enabled for this domain.
    #[serde(default)]
    pub tracking_enabled: Option<bool>,
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
    /// Why the record fails verification, when it does.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Request body for `create_domain`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateDomain {
    /// The domain name to add.
    pub domain: String,
    /// Proceed even if the domain's DNS is managed by another provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_conflicting_provider: Option<bool>,
    /// Enable bounce/complaint feedback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_enabled: Option<bool>,
    /// Deliver inbound mail for this domain to agent inboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_enabled: Option<bool>,
    /// Allow subdomains of this domain to be used for inboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomains_enabled: Option<bool>,
    /// Enable open/click tracking for this domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_enabled: Option<bool>,
}

/// Request body for `update_domain`. Fields left `None` stay unchanged.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDomain {
    /// Enable or disable bounce/complaint feedback.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback_enabled: Option<bool>,
    /// Enable or disable inbound delivery to agent inboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_enabled: Option<bool>,
    /// Enable or disable subdomains for inbox use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdomains_enabled: Option<bool>,
    /// Enable or disable open/click tracking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_enabled: Option<bool>,
}

/// The provider setup link for a domain, from `get_domain_setup_link`. When
/// [`DomainSetupLink::supported`] is false the registrar/provider flow isn't
/// available and DNS records must be configured manually.
#[derive(Clone, Debug, Deserialize)]
pub struct DomainSetupLink {
    /// Whether a provider setup flow exists for this domain.
    pub supported: bool,
    /// The detected DNS provider, when known.
    #[serde(default)]
    pub provider_name: Option<String>,
    /// The setup URL to open.
    #[serde(default)]
    pub url: Option<String>,
    /// Rendered QR-code width, when the link is meant to be scanned.
    #[serde(default)]
    pub width: Option<u32>,
    /// Rendered QR-code height.
    #[serde(default)]
    pub height: Option<u32>,
    /// Opaque state token carried through the provider flow.
    #[serde(default)]
    pub state: Option<String>,
    /// The conflicting provider, when setup is blocked by one.
    #[serde(default)]
    pub conflicting_provider: Option<String>,
}

/// One page of domains from `list_domains_page`.
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
