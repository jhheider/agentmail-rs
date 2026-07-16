use serde::Deserialize;

/// An entry in an inbox's audit log (label changes, deliveries, etc.).
#[derive(Clone, Debug, Deserialize)]
pub struct InboxEvent {
    /// Unique event id.
    pub event_id: String,
    /// The kind of event.
    #[serde(default)]
    pub event_type: Option<String>,
    /// The inbox the event belongs to.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// The organization the event belongs to.
    #[serde(default)]
    pub organization_id: Option<String>,
    /// The pod the event belongs to, when applicable.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// The message the event concerns, when applicable.
    #[serde(default)]
    pub message_id: Option<String>,
    /// The label involved, for label events.
    #[serde(default)]
    pub label: Option<String>,
    /// When the event occurred (RFC 3339).
    #[serde(default)]
    pub event_at: Option<String>,
    /// When the event record was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One page of inbox events from `list_inbox_events_page`.
#[derive(Clone, Debug, Deserialize)]
pub struct InboxEventList {
    /// Total events for the inbox (not just this page).
    pub count: u64,
    /// This page of events.
    #[serde(default)]
    pub events: Vec<InboxEvent>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
