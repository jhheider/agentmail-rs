use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/webhooks, subscribe an HTTPS endpoint to events
    /// (e.g. `message.received`). The response carries the signing `secret`
    /// exactly once; store it.
    pub async fn create_webhook(&self, webhook: CreateWebhook) -> Result<Webhook, Error> {
        self.request(reqwest::Method::POST, "/v0/webhooks", &[], Some(&webhook))
            .await
    }

    /// GET /v0/webhooks (first page; see [`Client::list_webhooks_page`]).
    pub async fn list_webhooks(&self) -> Result<WebhookList, Error> {
        self.list_webhooks_page(Page::default()).await
    }

    /// GET /v0/webhooks with pagination. Feed [`WebhookList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_webhooks_page(&self, page: Page) -> Result<WebhookList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/webhooks",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// DELETE /v0/webhooks/{webhook_id}
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/webhooks/{}", urlish(webhook_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
