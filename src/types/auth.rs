use serde::Deserialize;

/// Current identity from [`Client::auth_me`](crate::Client::auth_me).
///
/// Useful for scoped API keys that need to discover their parent
/// organization, pod, or inbox at runtime.
#[derive(Clone, Debug, Deserialize)]
pub struct Identity {
    /// What kind of credential (e.g. `api_key`, `pod_key`, `inbox_key`).
    #[serde(default)]
    pub credential_type: Option<String>,
    /// Organization id (always present).
    pub organization_id: String,
    /// Pod id (present when scoped to a pod).
    #[serde(default)]
    pub pod_id: Option<String>,
    /// Inbox id (present when scoped to an inbox).
    #[serde(default)]
    pub inbox_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_deserializes() {
        let id: Identity = serde_json::from_str(
            r#"{"credential_type":"api_key",
            "organization_id":"org_1"}"#,
        )
        .unwrap();
        assert_eq!(id.credential_type.as_deref(), Some("api_key"));
        assert_eq!(id.organization_id, "org_1");
        assert!(id.pod_id.is_none());
        assert!(id.inbox_id.is_none());

        // Scoped key.
        let id: Identity = serde_json::from_str(
            r#"{"credential_type":"inbox_key",
            "organization_id":"org_1","inbox_id":"in_1"}"#,
        )
        .unwrap();
        assert_eq!(id.inbox_id.as_deref(), Some("in_1"));
    }
}
