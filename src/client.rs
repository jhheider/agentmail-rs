use crate::{types::*, util::urlish, Error};

impl super::Client {
    // ── shared request machinery ─────────────────────────────────────────

    pub(crate) async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<serde_json::Value>,
    ) -> Result<T, Error> {
        let mut req = self
            .http
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.api_key);
        if !query.is_empty() {
            req = req.query(query);
        }
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(Error::Api { status, body: text });
        }
        // DELETE endpoints answer with an empty body; map that to null so
        // `()` (unit) deserializes.
        let text = if text.trim().is_empty() {
            "null"
        } else {
            &text
        };
        serde_json::from_str(text).map_err(|e| Error::Decode {
            reason: e.to_string(),
            body: text.to_string(),
        })
    }

    // ── Inboxes ──────────────────────────────────────────────────────────

    /// POST /v0/inboxes, a new agent-owned email address. Free plans get
    /// `{username}@agentmail.to`; custom domains must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/inboxes",
            &[],
            Some(serde_json::to_value(inbox).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes (first page; see `list_inboxes_page`).
    pub async fn list_inboxes(&self) -> Result<InboxList, Error> {
        self.list_inboxes_page(Page::default()).await
    }

    /// GET /v0/inboxes with pagination. Feed [`InboxList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_inboxes_page(&self, page: Page) -> Result<InboxList, Error> {
        self.request(reqwest::Method::GET, "/v0/inboxes", &page.query(), None)
            .await
    }

    /// GET /v0/inboxes/{inbox_id}
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            None,
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            None,
        )
        .await
    }

    // ── Messages ─────────────────────────────────────────────────────────

    /// POST /v0/inboxes/{inbox_id}/messages/send
    pub async fn send_message(
        &self,
        inbox_id: &str,
        message: SendMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/messages/send", urlish(inbox_id)),
            &[],
            Some(serde_json::to_value(message).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages (first page; see `list_messages_page`).
    pub async fn list_messages(&self, inbox_id: &str) -> Result<MessageList, Error> {
        self.list_messages_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/messages with pagination.
    pub async fn list_messages_page(
        &self,
        inbox_id: &str,
        page: Page,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &page.query(),
            None,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}
    pub async fn get_message(&self, inbox_id: &str, message_id: &str) -> Result<Message, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            None,
        )
        .await
    }

    // ── Webhooks ─────────────────────────────────────────────────────────

    /// POST /v0/webhooks, subscribe an HTTPS endpoint to events
    /// (e.g. `message.received`). The response carries the signing `secret`
    /// exactly once; store it.
    pub async fn create_webhook(&self, webhook: CreateWebhook) -> Result<Webhook, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/webhooks",
            &[],
            Some(serde_json::to_value(webhook).expect("serializable")),
        )
        .await
    }

    /// GET /v0/webhooks (first page; see `list_webhooks_page`).
    pub async fn list_webhooks(&self) -> Result<WebhookList, Error> {
        self.list_webhooks_page(Page::default()).await
    }

    /// GET /v0/webhooks with pagination.
    pub async fn list_webhooks_page(&self, page: Page) -> Result<WebhookList, Error> {
        self.request(reqwest::Method::GET, "/v0/webhooks", &page.query(), None)
            .await
    }

    /// DELETE /v0/webhooks/{webhook_id}
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/webhooks/{}", urlish(webhook_id)),
            &[],
            None,
        )
        .await
    }
}
