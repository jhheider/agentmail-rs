use serde::{Deserialize, Serialize};

use crate::util::QueryBuilder;

/// A human account (an app connection): who signed in to which app, through
/// which inbox, and when.
#[derive(Clone, Debug, Deserialize)]
pub struct Account {
    /// Unique account id (UUID).
    pub account_id: String,
    /// The app the account signed in to (UUID).
    pub app_id: String,
    /// The app's display name, when known.
    #[serde(default)]
    pub app_name: Option<String>,
    /// The inbox backing the connection.
    pub inbox_id: String,
    /// Owning pod.
    pub pod_id: String,
    /// Owning organization.
    pub organization_id: String,
    /// First sign-in (RFC 3339).
    pub first_signed_in_at: String,
    /// Most recent sign-in (RFC 3339).
    pub last_signed_in_at: String,
    /// Total sign-ins.
    pub sign_in_count: u64,
    /// `disabled`, when the account has been cut off.
    #[serde(default)]
    pub status: Option<String>,
    /// When the account was disabled (RFC 3339), when it was.
    #[serde(default)]
    pub disabled_at: Option<String>,
}

/// List filters for `list_accounts` (includes pagination, like the other
/// filter structs).
#[derive(Clone, Debug, Default)]
pub struct AccountsListFilters {
    /// Maximum accounts per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Sort by sign-in time, newest first.
    pub ascending: Option<bool>,
}

impl AccountsListFilters {
    pub(crate) fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .opt("ascending", self.ascending.as_ref())
            .build()
    }
}

/// Body for `update_account`. Fields left `None` stay unchanged.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateAccount {
    /// `enabled` to restore sign-ins, `disabled` to cut the account off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// One page of accounts from `list_accounts`.
#[derive(Clone, Debug, Deserialize)]
pub struct AccountList {
    /// Total accounts (not just this page).
    pub count: u64,
    /// This page of accounts.
    #[serde(default)]
    pub accounts: Vec<Account>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
