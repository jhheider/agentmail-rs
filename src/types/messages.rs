use serde::{Deserialize, Serialize};

/// Request body for `Client::send_message`. At least one recipient in `to`
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
}

/// The API's acknowledgement of a send.
#[derive(Clone, Debug, Deserialize)]
pub struct SentMessage {
    /// Id of the message just sent.
    pub message_id: String,
    /// Thread the message was filed under.
    pub thread_id: String,
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
}

/// One page of messages from `Client::list_messages_page`.
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
