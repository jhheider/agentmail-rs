use serde::{Deserialize, Serialize};

/// Which traffic direction a list governs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListDirection {
    /// Applied to outbound (sent) mail.
    Send,
    /// Applied to inbound (received) mail.
    Receive,
    /// Applied to replies.
    Reply,
    /// A direction this client version does not recognize.
    #[serde(other)]
    Unknown,
}

impl ListDirection {
    /// The path segment for this direction.
    pub(crate) fn as_path(self) -> &'static str {
        match self {
            ListDirection::Send => "send",
            ListDirection::Receive => "receive",
            ListDirection::Reply => "reply",
            ListDirection::Unknown => "unknown",
        }
    }
}

/// Whether a list allows or blocks its entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListKind {
    /// An allow list.
    Allow,
    /// A block list.
    Block,
    /// A kind this client version does not recognize.
    #[serde(other)]
    Unknown,
}

impl ListKind {
    /// The path segment for this kind.
    pub(crate) fn as_path(self) -> &'static str {
        match self {
            ListKind::Allow => "allow",
            ListKind::Block => "block",
            ListKind::Unknown => "unknown",
        }
    }
}

/// Whether a list entry is an address or a whole domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    /// A single email address.
    Email,
    /// A whole domain.
    Domain,
    /// An entry type this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// A single allow/block list entry.
#[derive(Clone, Debug, Deserialize)]
pub struct ListEntry {
    /// The address or domain.
    pub entry: String,
    /// Free-text reason recorded with the entry.
    #[serde(default)]
    pub reason: Option<String>,
    /// Whether the entry is an address or a domain.
    #[serde(default = "unknown_entry_type")]
    pub entry_type: EntryType,
    /// The organization the entry belongs to.
    #[serde(default)]
    pub organization_id: Option<String>,
    /// Whether the entry is read-only (system-managed).
    #[serde(default)]
    pub read_only: bool,
    /// Timestamp the entry was created (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}

fn unknown_entry_type() -> EntryType {
    EntryType::Unknown
}

/// Request body for [`Client::create_list_entry`](crate::Client::create_list_entry).
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateListEntry {
    /// The address or domain to add.
    pub entry: String,
    /// An optional reason to record with the entry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One page of list entries from [`Client::list_list_entries_page`](crate::Client::list_list_entries_page).
#[derive(Clone, Debug, Deserialize)]
pub struct ListEntries {
    /// Total entries in the list (not just this page).
    pub count: u64,
    /// This page of entries.
    #[serde(default)]
    pub entries: Vec<ListEntry>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
