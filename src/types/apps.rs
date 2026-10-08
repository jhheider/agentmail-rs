use serde::{Deserialize, Serialize};

use crate::util::QueryBuilder;

/// An app that can connect to AgentMail inboxes, from the app directory.
#[derive(Clone, Debug, Deserialize)]
pub struct App {
    /// Unique app id (UUID).
    pub app_id: String,
    /// URL slug, e.g. `my-agent-app`.
    #[serde(default)]
    pub slug: Option<String>,
    /// Display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Last update (RFC 3339), when set.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Marketing description.
    #[serde(default)]
    pub description: Option<String>,
    /// Logo image URL.
    #[serde(default)]
    pub logo_url: Option<String>,
    /// Terms-of-service URL.
    #[serde(default)]
    pub terms_url: Option<String>,
    /// Privacy-policy URL.
    #[serde(default)]
    pub privacy_url: Option<String>,
    /// Directory categories, e.g. `ai`, `productivity`.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Signups this app's owner may create, when capped.
    #[serde(default)]
    pub owner_signup_limit: Option<u32>,
}

/// List filters for `list_apps` (includes pagination).
#[derive(Clone, Debug, Default)]
pub struct AppsListFilters {
    /// Maximum apps per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Only apps in this directory category.
    pub category: Option<String>,
}

impl AppsListFilters {
    pub(crate) fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .opt("category", self.category.as_ref())
            .build()
    }
}

/// One page of apps from `list_apps`.
#[derive(Clone, Debug, Deserialize)]
pub struct AppList {
    /// Total apps (not just this page).
    pub count: u64,
    /// This page of apps.
    #[serde(default)]
    pub apps: Vec<App>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// One page of apps from `search_apps` (no pagination cursor).
#[derive(Clone, Debug, Deserialize)]
pub struct AppSearchList {
    /// Total matching apps.
    pub count: u64,
    /// The matching apps.
    #[serde(default)]
    pub apps: Vec<App>,
}

/// Body for `connect_app`: start a human's connection to an app. The human
/// opens [`ConnectAppResult::magic_url`] and approves; the API key id in the
/// result is the connection's credential handle.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ConnectApp {
    /// The inbox to connect; omitted means the whole organization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbox_id: Option<String>,
    /// Confirm the app's disclosure on the human's behalf.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_disclosure: Option<bool>,
}

/// The response to `connect_app`.
#[derive(Clone, Debug, Deserialize)]
pub struct ConnectAppResult {
    /// The API key created for this connection.
    pub api_key_id: String,
    /// The URL the human opens to approve the connection.
    pub magic_url: String,
    /// When `magic_url` expires (RFC 3339).
    pub expires_at: String,
}

/// One page of an app's accounts from `list_app_accounts`.
#[derive(Clone, Debug, Deserialize)]
pub struct AppAccountList {
    /// The app, when resolved.
    #[serde(default)]
    pub app: Option<App>,
    /// Total accounts for the app (not just this page).
    pub count: u64,
    /// This page of accounts.
    #[serde(default)]
    pub accounts: Vec<crate::Account>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
