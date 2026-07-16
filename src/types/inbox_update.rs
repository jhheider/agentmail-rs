use serde::Serialize;

/// Request body for [`Client::update_inbox`](crate::Client::update_inbox).
///
/// All fields are optional — only the fields you set will be patched.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateInbox {
    /// New display name (human-readable sender name).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Replace the stored metadata JSON blob.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
