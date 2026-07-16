use serde::{Deserialize, Serialize};

/// A pod -- an organizational container for inboxes, domains,
/// and webhooks with its own billing.
#[derive(Clone, Debug, Deserialize)]
pub struct Pod {
    /// Unique pod id.
    pub pod_id: String,
    /// Display name for the pod.
    #[serde(default)]
    pub name: Option<String>,
    /// Creator-provided client id (slug).
    #[serde(default)]
    pub client_id: Option<String>,
    /// The pod's plan tier.
    #[serde(default)]
    pub plan: Option<String>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Request body for `Client::create_pod`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreatePod {
    /// Short slug for the pod (e.g. `my-team`).
    pub client_id: String,
    /// Optional human-readable name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional billing plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
}

/// One page of pods from `Client::list_pods_page`.
#[derive(Clone, Debug, Deserialize)]
pub struct PodList {
    /// Total pods in the organization (not just this page).
    pub count: u64,
    /// This page of pods.
    #[serde(default)]
    pub pods: Vec<Pod>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pod_deserializes() {
        let p: Pod = serde_json::from_str(
            r#"{"pod_id":"pod_1","name":"My Pod",
            "client_id":"my-pod","plan":"free",
            "created_at":"2026-01-01T00:00:00Z"}"#,
        )
        .unwrap();
        assert_eq!(p.pod_id, "pod_1");
        assert_eq!(p.name.as_deref(), Some("My Pod"));
        assert_eq!(p.client_id.as_deref(), Some("my-pod"));
        assert_eq!(p.plan.as_deref(), Some("free"));
    }

    #[test]
    fn pod_list_deserializes() {
        let list: PodList = serde_json::from_str(
            r#"{"count":1,"pods":[{"pod_id":"pod_1"}],
            "next_page_token":null}"#,
        )
        .unwrap();
        assert_eq!(list.count, 1);
        assert_eq!(list.pods.len(), 1);
        assert_eq!(list.next_page_token, None);
    }
}
