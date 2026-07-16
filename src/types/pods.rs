use serde::{Deserialize, Serialize};

/// A pod, a container that owns inboxes and other resources.
#[derive(Clone, Debug, Deserialize)]
pub struct Pod {
    /// Unique pod id.
    pub pod_id: String,
    /// Human-readable pod name.
    #[serde(default)]
    pub name: Option<String>,
    /// Your reference id from creation, when set.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Timestamp the pod was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Timestamp the pod was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Request body for [`Client::create_pod`]. All fields optional.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreatePod {
    /// Human-readable pod name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Your own idempotency/reference id for this pod.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

/// One page of pods from [`Client::list_pods_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct PodList {
    /// Total pods in the account (not just this page).
    pub count: u64,
    /// This page of pods.
    #[serde(default)]
    pub pods: Vec<Pod>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
