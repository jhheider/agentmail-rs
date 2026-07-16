use crate::{Error, Page, types::*};

impl super::Client {
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
            &format!("/v0/webhooks/{}", crate::util::urlish(webhook_id)),
            &[],
            None,
        )
        .await
    }
}
