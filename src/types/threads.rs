use serde::{Deserialize, Serialize};

use super::Message;

// ── ThreadListFilters ─────────────────────────────────────────────────────

/// Filters for `Client::list_threads_page` and `Client::search_threads_page`.
#[derive(Clone, Debug, Default)]
pub struct ThreadListFilters {
    pub limit: Option<u32>,
    pub page_token: Option<String>,
    pub labels: Vec<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ascending: Option<bool>,
    pub include_spam: Option<bool>,
    pub include_blocked: Option<bool>,
    pub include_unauthenticated: Option<bool>,
    pub include_trash: Option<bool>,
    pub senders: Vec<String>,
    pub recipients: Vec<String>,
    pub subject: Vec<String>,
}

impl ThreadListFilters {
    /// Build query pairs for appending to a URL.
    pub fn query(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(limit) = self.limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(token) = &self.page_token {
            q.push(("page_token", token.clone()));
        }
        for label in &self.labels {
            q.push(("labels", label.clone()));
        }
        if let Some(before) = &self.before {
            q.push(("before", before.clone()));
        }
        if let Some(after) = &self.after {
            q.push(("after", after.clone()));
        }
        if let Some(asc) = self.ascending {
            q.push(("ascending", asc.to_string()));
        }
        if let Some(sp) = self.include_spam {
            q.push(("include_spam", sp.to_string()));
        }
        if let Some(bl) = self.include_blocked {
            q.push(("include_blocked", bl.to_string()));
        }
        if let Some(ua) = self.include_unauthenticated {
            q.push(("include_unauthenticated", ua.to_string()));
        }
        if let Some(tr) = self.include_trash {
            q.push(("include_trash", tr.to_string()));
        }
        for sender in &self.senders {
            q.push(("senders", sender.clone()));
        }
        for recip in &self.recipients {
            q.push(("recipients", recip.clone()));
        }
        for subj in &self.subject {
            q.push(("subject", subj.clone()));
        }
        q
    }
}

// ── Thread ────────────────────────────────────────────────────────────────

/// A full thread, including its messages and attachments.
#[derive(Clone, Debug, Deserialize)]
pub struct Thread {
    pub thread_id: String,
    #[serde(default)]
    pub inbox_id: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub senders: Vec<String>,
    #[serde(default)]
    pub recipients: Vec<String>,
    #[serde(default)]
    pub message_count: Option<u64>,
    #[serde(default)]
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

// ── ThreadItem ────────────────────────────────────────────────────────────

/// A lightweight thread representation returned in list/search responses
/// when the full thread body is not needed.
#[derive(Clone, Debug, Deserialize)]
pub struct ThreadItem {
    pub thread_id: String,
    #[serde(default)]
    pub inbox_id: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub senders: Vec<String>,
    #[serde(default)]
    pub recipients: Vec<String>,
}

// ── ThreadList ────────────────────────────────────────────────────────────

/// One page of threads from `Client::list_threads_page` / `Client::search_threads_page`.
#[derive(Clone, Debug, Deserialize)]
pub struct ThreadList {
    /// Total threads matching the query (not just this page).
    pub count: u64,
    /// Threads on this page.
    #[serde(default)]
    pub threads: Vec<Thread>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

// ── UpdateThread ──────────────────────────────────────────────────────────

/// Request body for `Client::update_thread`.
///
/// Only the labels you explicitly add or remove are changed.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateThread {
    /// Labels to add to the thread.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub add_labels: Vec<String>,
    /// Labels to remove from the thread.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remove_labels: Vec<String>,
}

// ── Attachment ────────────────────────────────────────────────────────────

/// A file attached to a thread.
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    pub attachment_id: String,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thread_search_deserializes() {
        let list: ThreadList = serde_json::from_str(
            r#"{"count":2,"threads":[
            {"thread_id":"t1","subject":"Hello","message_count":3},
            {"thread_id":"t2","subject":"World","message_count":1}
            ]}"#,
        )
        .unwrap();
        assert_eq!(list.count, 2);
        assert_eq!(list.threads.len(), 2);
        assert_eq!(list.threads[0].subject.as_deref(), Some("Hello"));
        assert_eq!(list.threads[1].message_count, Some(1));
    }

    #[test]
    fn thread_full_get_deserializes() {
        let thread: Thread = serde_json::from_str(
            r#"{"thread_id":"t1","inbox_id":"ib_1","subject":"Hello",
            "labels":["unread"],"senders":["a@b.c"],"recipients":["x@y.z"],
            "message_count":3,"last_message_at":"2026-01-15T09:30:00Z",
            "created_at":"2026-01-15T09:30:00Z",
            "messages":[{"message_id":"m1","thread_id":"t1","from":"a@b.c",
                         "subject":"Hello","preview":"...",
                         "timestamp":"2026-01-15T09:30:00Z"}],
            "snippet":"Hello world",
            "attachments":[{"attachment_id":"a1","filename":"doc.pdf",
                            "content_type":"application/pdf","size":1234}]}"#,
        )
        .unwrap();
        assert_eq!(thread.thread_id, "t1");
        assert_eq!(thread.inbox_id.as_deref(), Some("ib_1"));
        assert_eq!(thread.messages.len(), 1);
        assert_eq!(thread.messages[0].message_id, "m1");
        assert_eq!(thread.attachments.len(), 1);
        assert_eq!(thread.attachments[0].filename.as_deref(), Some("doc.pdf"));
    }

    #[test]
    fn thread_item_deserializes() {
        let item: ThreadItem = serde_json::from_str(
            r#"{"thread_id":"t1","inbox_id":"ib_1","subject":"Hi",
            "labels":["unread"],"senders":["a@b.c"],"recipients":["x@y.z"]}"#,
        )
        .unwrap();
        assert_eq!(item.thread_id, "t1");
        assert_eq!(item.labels, vec!["unread"]);
    }

    #[test]
    fn update_thread_serializes() {
        let body = serde_json::to_value(UpdateThread {
            add_labels: vec!["starred".into()],
            ..Default::default()
        })
        .unwrap();
        assert_eq!(body, serde_json::json!({"add_labels":["starred"]}));

        let body = serde_json::to_value(UpdateThread {
            add_labels: vec!["starred".into()],
            remove_labels: vec!["unread".into()],
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"add_labels":["starred"],"remove_labels":["unread"]}),
        );
    }

    #[test]
    fn thread_list_filters_query() {
        let f = ThreadListFilters::default();
        assert!(f.query().is_empty());

        let f = ThreadListFilters {
            limit: Some(10),
            page_token: Some("tok".into()),
            labels: vec!["unread".into()],
            include_spam: Some(true),
            ..Default::default()
        };
        let q = f.query();
        assert!(q.contains(&("limit", "10".to_string())));
        assert!(q.contains(&("page_token", "tok".to_string())));
        assert!(q.contains(&("labels", "unread".to_string())));
        assert!(q.contains(&("include_spam", "true".to_string())));
    }
}
