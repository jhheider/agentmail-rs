use serde::{Deserialize, Serialize};

/// Request body for [`Client::create_inbox`]. All fields optional; the API
/// generates a username when none is given.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateInbox {
    /// Local part of the address; random when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// A verified domain (or subdomain of one); defaults to `agentmail.to`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Human-readable sender name shown in email clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Your own idempotency/reference id for this inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Arbitrary JSON stored alongside the inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
/// An agent-owned inbox, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Inbox {
    /// Unique id, used in every message call.
    pub inbox_id: String,
    /// The address itself, e.g. `my-agent@agentmail.to`.
    pub email: String,
    /// Human-readable sender name, when set.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Owning pod, when the account uses pods.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// Your reference id from creation, when set.
    #[serde(default)]
    pub client_id: Option<String>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}
/// One page of inboxes from [`Client::list_inboxes_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct InboxList {
    /// Total inboxes in the account (not just this page).
    pub count: u64,
    /// This page of inboxes.
    #[serde(default)]
    pub inboxes: Vec<Inbox>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
