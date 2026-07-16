use crate::util::QueryBuilder;
use serde::{Deserialize, Serialize};

use super::{Attachment, Message};

/// A conversation thread. List items omit `messages`; the get-thread shape
/// includes them and search results carry `highlights`; every optional field
/// defaults so all three parse into this one type.
#[derive(Clone, Debug, Deserialize)]
pub struct Thread {
    /// Unique thread id.
    pub thread_id: String,
    /// Inbox the thread belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Labels on the thread.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Timestamp of the most recent activity (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Timestamp of the most recent received message (RFC 3339).
    #[serde(default)]
    pub received_timestamp: Option<String>,
    /// Timestamp of the most recent sent message (RFC 3339).
    #[serde(default)]
    pub sent_timestamp: Option<String>,
    /// Distinct sender addresses across the thread.
    #[serde(default)]
    pub senders: Vec<String>,
    /// Distinct recipient addresses across the thread.
    #[serde(default)]
    pub recipients: Vec<String>,
    /// Subject line.
    #[serde(default)]
    pub subject: Option<String>,
    /// Short preview of the latest message.
    #[serde(default)]
    pub preview: Option<String>,
    /// Attachments across the thread.
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Id of the most recent message.
    #[serde(default)]
    pub last_message_id: Option<String>,
    /// Number of messages in the thread.
    #[serde(default)]
    pub message_count: Option<u64>,
    /// Total size of the thread in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// Timestamp the thread was last updated (RFC 3339).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Timestamp the thread was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Full messages, present in get-thread responses.
    #[serde(default)]
    pub messages: Vec<Message>,
    /// Search-highlight fragments, present in search responses.
    #[serde(default)]
    pub highlights: Option<serde_json::Value>,
}

/// One page of threads from a list or search call.
#[derive(Clone, Debug, Deserialize)]
pub struct ThreadList {
    /// Total threads matching the query (not just this page).
    pub count: u64,
    /// This page of threads.
    #[serde(default)]
    pub threads: Vec<Thread>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// Request body for `update_thread`: labels to add and/or remove.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateThread {
    /// Labels to add.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}

/// The API's response to `update_thread`: the thread id and its
/// labels after the update.
#[derive(Clone, Debug, Deserialize)]
pub struct UpdatedThread {
    /// Id of the updated thread.
    pub thread_id: String,
    /// The thread's labels after the update.
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Filters for `list_threads_filtered` and
/// `search_threads_page`. Pagination fields (`limit`, `page_token`)
/// share the same query-parameter namespace as the filters.
#[derive(Clone, Debug, Default)]
pub struct ThreadListFilters {
    /// Maximum items per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Filter by labels.
    pub labels: Vec<String>,
    /// Only threads before this timestamp (RFC 3339).
    pub before: Option<String>,
    /// Only threads after this timestamp (RFC 3339).
    pub after: Option<String>,
    /// Return oldest first instead of newest first.
    pub ascending: Option<bool>,
    /// Include spam threads.
    pub include_spam: Option<bool>,
    /// Include blocked threads.
    pub include_blocked: Option<bool>,
    /// Include unauthenticated threads.
    pub include_unauthenticated: Option<bool>,
    /// Include trashed threads.
    pub include_trash: Option<bool>,
    /// Filter by sender substring (repeatable).
    pub senders: Vec<String>,
    /// Filter by recipient substring (repeatable).
    pub recipients: Vec<String>,
    /// Filter by subject substring (repeatable).
    pub subject: Vec<String>,
}

impl ThreadListFilters {
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
            .many("senders", &self.senders)
            .many("recipients", &self.recipients)
            .many("subject", &self.subject)
            .build()
    }
}
