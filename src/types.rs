use serde::{Deserialize, Serialize};

// ─── Page ─────────────────────────────────────────────────────────────────

/// Pagination controls for the `list_*_page` calls. `Default` is the API's
/// own defaults (first page, server-chosen page size).
#[derive(Clone, Debug, Default)]
pub struct Page {
    /// Maximum items per page (the API caps this server-side).
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
}

impl Page {
    /// Build query pairs for appending to a URL.
    pub fn query(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(limit) = self.limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(token) = &self.page_token {
            q.push(("page_token", token.clone()));
        }
        q
    }
}

// ─── Inboxes ───────────────────────────────────────────────────────────────

/// Request body for `Client::create_inbox`. All fields optional; the API
/// generates a username when none is given.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateInbox {
    /// Local part of the address; random when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// A verified domain (or subdomain of one); defaults to `agentmail.to`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    /// Human-readable sender name shown in email clients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Your own idempotency/reference id for this inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Arbitrary JSON stored alongside the inbox.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// An agent-owned inbox, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct Inbox {
    /// Unique id, used in every message call.
    pub inbox_id: String,
    /// The address itself, e.g. `my-agent@agentmail.to`.
    pub email: String,
    /// Human-readable sender name, when set.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Owning pod, when the account uses pods.
    #[serde(default)]
    pub pod_id: Option<String>,
    /// Your reference id from creation, when set.
    #[serde(default)]
    pub client_id: Option<String>,
    /// RFC 3339 creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
}

/// One page of inboxes from `Client::list_inboxes_page`.
#[derive(Clone, Debug, Deserialize)]
pub struct InboxList {
    /// Total inboxes in the account (not just this page).
    pub count: u64,
    /// This page of inboxes.
    #[serde(default)]
    pub inboxes: Vec<Inbox>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

// ─── Messages ──────────────────────────────────────────────────────────────

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

// ─── Webhooks ──────────────────────────────────────────────────────────────

/// Request body for `Client::create_webhook`.
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

/// One page of webhooks from `Client::list_webhooks_page`.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shapes_round_trip() {
        let inbox: Inbox = serde_json::from_str(
            r#"{"pod_id":"pod_1","inbox_id":"ib_1","email":"x@agentmail.to",
                "display_name":"X","updated_at":"2024-01-15T09:30:00Z",
                "created_at":"2024-01-15T09:30:00Z","surprise_field":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.email, "x@agentmail.to");

        let list: MessageList = serde_json::from_str(
            r#"{"count":1,"messages":[{"message_id":"m1","thread_id":"t1",
                "from":"a@b.c","subject":"hi","preview":"…","timestamp":"2026-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        assert_eq!(list.messages[0].message_id, "m1");
        assert!(list.messages[0].text.is_none());

        let body = serde_json::to_value(SendMessage {
            to: vec!["a@b.c".into()],
            subject: Some("s".into()),
            text: Some("t".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"to":["a@b.c"],"subject":"s","text":"t"}),
        );
    }
}
