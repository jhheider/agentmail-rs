use crate::util::QueryBuilder;
use serde::{Deserialize, Serialize};

use super::{Attachment, SendAttachment};

/// Request body for [`Client::send_message`](crate::Client::send_message). At least one recipient in `to`
/// and one of `text`/`html` are required by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SendMessage {
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
    /// HTML body (send both for multipart).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to attach to the sent message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    /// Reply-To addresses.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reply_to: Vec<String>,
    /// Files to attach.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<SendAttachment>,
    /// Extra headers to set on the outgoing message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
}
/// The API's acknowledgement of a send.
#[derive(Clone, Debug, Deserialize)]
pub struct SentMessage {
    /// Id of the message just sent.
    pub message_id: String,
    /// Thread the message was filed under.
    pub thread_id: String,
}
/// Request body for [`Client::reply_to_message`](crate::Client::reply_to_message) and
/// [`Client::reply_all_to_message`](crate::Client::reply_all_to_message). The `to` field is derived from the
/// parent message; at least one of `text`/`html` is required by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ReplyToMessage {
    /// Carbon-copy recipients (in addition to those on the thread).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<String>,
    /// Plain-text body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
    /// Labels to attach to the reply.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    /// Files to attach.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<SendAttachment>,
    /// Extra headers to set on the outgoing message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<serde_json::Value>,
}

/// The presigned download for a message's raw RFC 822 source, from
/// [`Client::get_raw_message`](crate::Client::get_raw_message). Fetch the bytes with [`Client::download_raw`](crate::Client::download_raw).
#[derive(Clone, Debug, Deserialize)]
pub struct RawMessage {
    /// The message id.
    pub message_id: String,
    /// Size of the raw message in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// Short-lived presigned URL to download the `.eml` bytes.
    pub download_url: String,
    /// When `download_url` expires (RFC 3339).
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Request body for [`Client::batch_get_messages`](crate::Client::batch_get_messages).
#[derive(Clone, Debug, Default, Serialize)]
pub struct BatchGetMessages {
    /// The message ids to fetch.
    pub message_ids: Vec<String>,
}

/// Response to [`Client::batch_get_messages`](crate::Client::batch_get_messages).
#[derive(Clone, Debug, Deserialize)]
pub struct BatchGetMessagesResponse {
    /// The number of messages returned.
    pub count: u64,
    /// The requested messages.
    #[serde(default)]
    pub messages: Vec<Message>,
}

/// Request body for [`Client::batch_update_messages`](crate::Client::batch_update_messages): apply the same label
/// changes to many messages at once.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BatchUpdateMessages {
    /// The message ids to update.
    pub message_ids: Vec<String>,
    /// Labels to add to each message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove from each message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}

/// Response to [`Client::batch_update_messages`](crate::Client::batch_update_messages).
#[derive(Clone, Debug, Deserialize)]
pub struct BatchUpdateMessagesResponse {
    /// The number of messages updated.
    pub count: u64,
    /// The per-message results.
    #[serde(default)]
    pub updates: Vec<UpdatedMessage>,
}
/// Request body for [`Client::update_message`](crate::Client::update_message).
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateMessage {
    /// Labels to add.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}
/// The API's response to [`Client::update_message`](crate::Client::update_message): the message id and its
/// labels after the update.
#[derive(Clone, Debug, Deserialize)]
pub struct UpdatedMessage {
    /// Id of the updated message.
    pub message_id: String,
    /// The message's labels after the update.
    #[serde(default)]
    pub labels: Vec<String>,
}
/// A message as the API returns it. List items are a subset of the full
/// get-message shape; every non-id field defaults so both parse.
#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    /// Unique id within the inbox.
    pub message_id: String,
    /// Inbox the message belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Conversation thread id.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// Sender address.
    #[serde(default)]
    pub from: Option<String>,
    /// Recipient addresses.
    #[serde(default)]
    pub to: Vec<String>,
    /// Subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Short plain-text excerpt (list responses).
    #[serde(default)]
    pub preview: Option<String>,
    /// Full plain-text body (get responses).
    #[serde(default)]
    pub text: Option<String>,
    /// Full HTML body (get responses).
    #[serde(default)]
    pub html: Option<String>,
    /// Labels on the message (e.g. `received`, `unread`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// RFC 3339 send/receive timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Attachments on the message (present in get-message responses).
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}
/// One page of messages from [`Client::list_messages_page`](crate::Client::list_messages_page).
#[derive(Clone, Debug, Deserialize)]
pub struct MessageList {
    /// Total messages in the inbox (not just this page).
    pub count: u64,
    /// This page of messages.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
/// Filters for [`Client::list_messages_filtered`](crate::Client::list_messages_filtered) and
/// [`Client::search_messages_page`](crate::Client::search_messages_page). Pagination fields (`limit`,
/// `page_token`) live here because they share the same query-parameter
/// namespace as the filter fields.
#[derive(Clone, Debug, Default)]
pub struct MessageListFilters {
    /// Maximum items per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Filter by labels.
    pub labels: Vec<String>,
    /// Only messages before this timestamp (RFC 3339).
    pub before: Option<String>,
    /// Only messages after this timestamp (RFC 3339).
    pub after: Option<String>,
    /// Return oldest first instead of newest first.
    pub ascending: Option<bool>,
    /// Include spam messages.
    pub include_spam: Option<bool>,
    /// Include blocked messages.
    pub include_blocked: Option<bool>,
    /// Include unauthenticated messages.
    pub include_unauthenticated: Option<bool>,
    /// Include trashed messages.
    pub include_trash: Option<bool>,
    /// Filter by sender substring (repeatable).
    pub from: Vec<String>,
    /// Filter by recipient substring -- matches to, cc, or bcc (repeatable).
    pub to: Vec<String>,
    /// Filter by subject substring (repeatable).
    pub subject: Vec<String>,
}
impl MessageListFilters {
    pub(crate) fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .many("labels", &self.labels)
            .opt("before", self.before.as_ref())
            .opt("after", self.after.as_ref())
            .opt("ascending", self.ascending.as_ref())
            .opt("include_spam", self.include_spam.as_ref())
            .opt("include_blocked", self.include_blocked.as_ref())
            .opt(
                "include_unauthenticated",
                self.include_unauthenticated.as_ref(),
            )
            .opt("include_trash", self.include_trash.as_ref())
            .many("from", &self.from)
            .many("to", &self.to)
            .many("subject", &self.subject)
            .build()
    }
}
