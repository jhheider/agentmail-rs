use serde::{Deserialize, Serialize};

/// A verified or unverified custom domain.
#[derive(Clone, Debug, Deserialize)]
pub struct Domain {
    /// Unique domain id.
    pub domain_id: String,
    /// The registered domain name (e.g. `example.com`).
    pub domain: String,
    /// Verification status: `pending`, `verified`, or `failed`.
    #[serde(default)]
    pub status: Option<String>,
    /// DNS records the owner must publish for verification.
    #[serde(default)]
    pub dns_records: Vec<DnsRecord>,
    /// Whether the MX records have been verified.
    #[serde(default)]
    pub mx_verified: Option<bool>,
    /// Whether DKIM signing is configured.
    #[serde(default)]
    pub dkim_verified: Option<bool>,
    /// Whether SPF is verified.
    #[serde(default)]
    pub spf_verified: Option<bool>,
    /// Whether DMARC is configured.
    #[serde(default)]
    pub dmarc_verified: Option<bool>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A DNS record required for domain verification.
#[derive(Clone, Debug, Deserialize)]
pub struct DnsRecord {
    /// Record type: `TXT`, `MX`, `CNAME`, etc.
    #[serde(default)]
    pub record_type: Option<String>,
    /// Hostname (name) for this record.
    #[serde(default)]
    pub name: Option<String>,
    /// Record value.
    #[serde(default)]
    pub value: Option<String>,
    /// TTL in seconds, if specified.
    #[serde(default)]
    pub ttl: Option<i64>,
}

/// Request body for [`Client::create_domain`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateDomain {
    /// The domain name to register (e.g. `example.com`).
    pub domain: String,
}

/// Request body for [`Client::update_domain`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDomain {
    /// Catch-all forwarding address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catch_all_forward: Option<String>,
    /// Catch-all SPF pass-through.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catch_all_spf: Option<String>,
    /// DMARC policy override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dmarc_policy: Option<String>,
}

/// One page of domains from [`Client::list_domains_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct DomainList {
    /// Total domains in the organization (not just this page).
    pub count: u64,
    /// This page of domains.
    #[serde(default)]
    pub domains: Vec<Domain>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_deserializes() {
        let d: Domain = serde_json::from_str(
            r#"{"domain_id":"dom_1","domain":"example.com",
            "status":"verified","dns_records":[],
            "created_at":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(d.domain_id, "dom_1");
        assert_eq!(d.domain, "example.com");
        assert_eq!(d.status.as_deref(), Some("verified"));
        assert!(d.dns_records.is_empty());
    }

    #[test]
    fn domain_list_deserializes() {
        let list: DomainList = serde_json::from_str(
            r#"{"count":1,"domains":[{"domain_id":"dom_1",
            "domain":"example.com"}],
            "next_page_token":null}"#,
        )
        .unwrap();
        assert_eq!(list.count, 1);
        assert_eq!(list.domains.len(), 1);
        assert_eq!(list.next_page_token, None);
    }
}
