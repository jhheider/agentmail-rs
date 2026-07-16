use serde::{Deserialize, Serialize};

/// Request body for [`Client::create_webhook`].
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateWebhook {
    /// HTTPS endpoint to deliver events to.
    pub url: String,
    /// e.g. `["message.received"]`.
    pub event_types: Vec<String>,
    /// Limit delivery to these inboxes; empty means all.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inbox_ids: Vec<String>,
    /// Your own idempotency/reference id for this webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}
/// A webhook subscription, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Webhook {
    /// Unique id, used to delete the subscription.
    pub webhook_id: String,
    /// The subscribed HTTPS endpoint.
    pub url: String,
    /// Signing secret, returned once, on creation.
    #[serde(default)]
    pub secret: Option<String>,
    /// Subscribed event types.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// Inboxes the subscription is limited to; empty means all.
    #[serde(default)]
    pub inbox_ids: Vec<String>,
    /// Whether the subscription currently delivers.
    pub enabled: bool,
}
/// One page of webhooks from [`Client::list_webhooks_page`].
#[derive(Clone, Debug, Deserialize)]
pub struct WebhookList {
    /// Total webhooks in the account (not just this page).
    pub count: u64,
    /// This page of webhooks.
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}
