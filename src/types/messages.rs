use crate::util::QueryBuilder;
use serde::{Deserialize, Serialize};

use super::{Attachment, SendAttachment};

/// Request body for `send_message`. At least one recipient in `to`
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
    /// Ask the API to make open tracking available for this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_opens: Option<bool>,
}
/// The API's acknowledgement of a send.
#[derive(Clone, Debug, Deserialize)]
pub struct SentMessage {
    /// Id of the message just sent.
    pub message_id: String,
    /// Thread the message was filed under.
    pub thread_id: String,
}
/// Request body for `reply_to_message` and
/// `reply_all_to_message`. `to` overrides the derived recipients when
/// non-empty; at least one of `text`/`html` is required by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ReplyToMessage {
    /// Recipient override; empty means derive from the parent message.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    /// Treat the reply as a reply-all (server-side convenience flag for
    /// `reply_to_message`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_all: Option<bool>,
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
    /// Ask the API to make open tracking available for this reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track_opens: Option<bool>,
}

/// The presigned download for a message's raw RFC 822 source, from
/// `get_raw_message`. Fetch the bytes with `download_raw`.
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

/// Request body for `batch_get_messages`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct BatchGetMessages {
    /// The message ids to fetch.
    pub message_ids: Vec<String>,
}

/// Response to `batch_get_messages`.
#[derive(Clone, Debug, Deserialize)]
pub struct BatchGetMessagesResponse {
    /// The number of messages returned.
    pub count: u64,
    /// The requested messages.
    #[serde(default)]
    pub messages: Vec<Message>,
}

/// Request body for `batch_update_messages`: apply the same label
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

/// Response to `batch_update_messages`.
#[derive(Clone, Debug, Deserialize)]
pub struct BatchUpdateMessagesResponse {
    /// The number of messages updated.
    pub count: u64,
    /// The per-message results.
    #[serde(default)]
    pub updates: Vec<UpdatedMessage>,
}
/// Request body for `update_message`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateMessage {
    /// Labels to add.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}
/// The API's response to `update_message`: the message id and its
/// labels after the update.
#[derive(Clone, Debug, Deserialize)]
pub struct UpdatedMessage {
    /// Id of the updated message.
    pub message_id: String,
    /// The message's labels after the update.
    #[serde(default)]
    pub labels: Vec<String>,
}
/// A message as the API returns it. List and search items are a subset of the
/// full get-message shape; every non-id field defaults so all three parse.
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
    /// Addresses replied to, when set.
    #[serde(default)]
    pub reply_to: Vec<String>,
    /// Carbon-copy recipients.
    #[serde(default)]
    pub cc: Vec<String>,
    /// Blind-carbon-copy recipients.
    #[serde(default)]
    pub bcc: Vec<String>,
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
    /// Plain-text body extracted from attachments by the API.
    #[serde(default)]
    pub extracted_text: Option<String>,
    /// HTML body extracted from attachments by the API.
    #[serde(default)]
    pub extracted_html: Option<String>,
    /// Labels on the message (e.g. `received`, `unread`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// RFC 3339 send/receive timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Attachments on the message (present in get-message responses).
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// `Message-ID` of the message this one replies to, when set.
    #[serde(default)]
    pub in_reply_to: Option<String>,
    /// `References` chain of `Message-ID`s, when set.
    #[serde(default)]
    pub references: Vec<String>,
    /// Extra headers on the message, when set.
    #[serde(default)]
    pub headers: Option<serde_json::Value>,
    /// Calendar event attached to the message, when it carries an invite.
    #[serde(default)]
    pub calendar_event_id: Option<String>,
    /// Agent Armor security analysis of the message, when the feature is
    /// enabled for the organization (see [`AgentArmorResult`]).
    #[serde(default)]
    pub agent_armor: Option<AgentArmorResult>,
    /// Total message size in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// Timestamp the message was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Timestamp the message was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Search-highlight fragments, present in search responses.
    #[serde(default)]
    pub highlights: Option<serde_json::Value>,
}
/// One page of messages from `list_messages_page`.
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
/// Filters for `list_messages_filtered` and
/// `search_messages_page`. Pagination fields (`limit`,
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

/// Agent Armor's verdict on a message, from [`AgentArmorResult::verdict`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentArmorVerdict {
    /// No threat found.
    Clear,
    /// Flagged for human review.
    Review,
    /// A verdict this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Why Agent Armor could not analyze a message, from
/// [`AgentArmorResult::unavailable_reason`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentArmorUnavailableReason {
    /// The sender domain is not a deliverable DNS domain.
    UnsupportedDsn,
    /// The message exceeded the analysis size limit.
    PayloadTooLarge,
    /// No content the scorer can read (empty or image-only).
    NoScannableContent,
    /// The analysis queue rejected the job.
    EnqueueFailed,
    /// The scoring model failed.
    ScorerFailed,
    /// The model's answer violated the result contract.
    ContractError,
    /// Analysis did not finish within its deadline.
    DeadlineExceeded,
    /// A reason this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Agent Armor security analysis attached to a [`Message`], when the
/// organization has the feature enabled. While analysis is running only
/// `status` and the pending timestamps are set; afterwards exactly one of
/// `verdict` (with `reason` and `model`) or `unavailable_reason` is set.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentArmorResult {
    /// `pending`, `complete`, or `unavailable`.
    #[serde(default)]
    pub status: Option<String>,
    /// When analysis started (RFC 3339; pending results).
    #[serde(default)]
    pub started_at: Option<String>,
    /// When analysis times out (RFC 3339; pending results).
    #[serde(default)]
    pub deadline_at: Option<String>,
    /// The verdict (complete results).
    #[serde(default)]
    pub verdict: Option<AgentArmorVerdict>,
    /// Why the model flagged the message for review (complete results).
    #[serde(default)]
    pub reason: Option<String>,
    /// The scoring model that produced the verdict (complete results).
    #[serde(default)]
    pub model: Option<String>,
    /// When analysis finished (RFC 3339; complete results).
    #[serde(default)]
    pub completed_at: Option<String>,
    /// Why analysis produced no verdict (unavailable results).
    #[serde(default)]
    pub unavailable_reason: Option<AgentArmorUnavailableReason>,
}

/// What you think of an Agent Armor verdict, for
/// `create_agent_armor_feedback`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentArmorFeedbackValue {
    /// The `review` verdict was correct.
    CorrectReview,
    /// The `review` verdict was wrong; the message is fine.
    IncorrectReview,
    /// A threat got through without a `review` verdict.
    MissedThreat,
    /// The message was legitimate agent instruction, flagged anyway.
    LegitimateAgentInstruction,
    /// Spam or phishing.
    SpamOrPhishing,
    /// A value this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Request body for `create_agent_armor_feedback`.
#[derive(Clone, Debug, Serialize)]
pub struct CreateAgentArmorFeedback {
    /// Your assessment of the verdict.
    pub feedback: AgentArmorFeedbackValue,
    /// Free-form context for the AgentMail team.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Who left Agent Armor feedback: an API key or the console. The oneOf is
/// modeled flat; exactly one of the fields past `kind` is set.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentArmorFeedbackActor {
    /// `api_key` or `console`.
    #[serde(default)]
    pub kind: Option<String>,
    /// The API key that submitted the feedback (api_key actors).
    #[serde(default)]
    pub api_key_id: Option<String>,
}

/// A feedback submission, from `create_agent_armor_feedback`.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentArmorFeedback {
    /// Unique feedback id.
    pub feedback_id: String,
    /// The submitted assessment.
    pub feedback: AgentArmorFeedbackValue,
    /// Free-form context, when provided.
    #[serde(default)]
    pub notes: Option<String>,
    /// Who submitted the feedback.
    #[serde(default)]
    pub actor: Option<AgentArmorFeedbackActor>,
    /// The verdict the feedback is about.
    #[serde(default)]
    pub original_verdict: Option<AgentArmorVerdict>,
    /// The scoring model that produced that verdict.
    #[serde(default)]
    pub model: Option<String>,
    /// Timestamp the feedback was submitted (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}
