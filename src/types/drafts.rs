use super::Attachment;
use serde::{Deserialize, Serialize};

/// Request body for `create_draft`. At least one recipient or a
/// reply/forward-of reference and one of `text`/`html` are required by the
/// API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateDraft {
    /// Primary recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Message id this draft is in reply to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    /// When creating a reply draft, set true to reply to all recipients of
    /// the referenced message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_all: Option<bool>,
    /// Message id this draft is a forward of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_of: Option<String>,
    /// Labels to attach to the draft.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    /// Schedule send at this RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,
}
/// Request body for `update_draft`. Every field is optional;
/// omitted fields are left unchanged on the server. Pass `Some(vec![])`
/// to clear a recipient field; pass `None` to leave it alone.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateDraft {
    /// Primary recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<String>>,
    /// Carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<String>>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<String>>,
    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to add.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_labels: Option<Vec<String>>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_labels: Option<Vec<String>>,
    /// Schedule send at this RFC 3339 timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,
}
/// A draft message, as the API returns it. List items are a subset of the
/// full get-draft shape; every optional field defaults so both parse.
#[derive(Clone, Debug, Deserialize)]
pub struct Draft {
    /// Unique draft id.
    pub draft_id: String,
    /// Inbox the draft belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Primary recipients.
    #[serde(default)]
    pub to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(default)]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(default)]
    pub bcc: Vec<String>,
    /// Subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Plain-text body.
    #[serde(default)]
    pub text: Option<String>,
    /// HTML body.
    #[serde(default)]
    pub html: Option<String>,
    /// Labels on the draft.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Message id this draft is replying to.
    #[serde(default)]
    pub in_reply_to: Option<String>,
    /// Whether the draft was created as a reply-all.
    #[serde(default)]
    pub reply_all: Option<bool>,
    /// Message id this draft is forwarding.
    #[serde(default)]
    pub forward_of: Option<String>,
    /// Scheduled send time (RFC 3339).
    #[serde(default)]
    pub send_at: Option<String>,
    /// Timestamp the draft was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Timestamp the draft was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Sender address (available in get responses).
    #[serde(default)]
    pub from: Option<String>,
    /// Attachments on the draft.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}
/// One page of drafts from `list_drafts_page`.
#[derive(Clone, Debug, Deserialize)]
pub struct DraftList {
    /// Total drafts in the inbox (not just this page).
    pub count: u64,
    /// This page of drafts.
    #[serde(default)]
    pub drafts: Vec<Draft>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
