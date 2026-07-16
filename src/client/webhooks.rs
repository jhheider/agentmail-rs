use crate::client::NoBody;
use crate::client::scope::{Scoped, Webhooks};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Webhooks> Scoped<'_, S> {
    /// POST `{scope}/webhooks`, subscribe an HTTPS endpoint to events (e.g.
    /// `message.received`). The response carries the signing `secret` exactly
    /// once; store it. Inbox and pod scopes ignore `inbox_ids`/`pod_ids` (the
    /// scope already targets the delivery); set those only at [`Client::org`](crate::Client::org).
    pub async fn create_webhook(&self, webhook: CreateWebhook) -> Result<Webhook, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/webhooks", self.base()),
                &[],
                Some(&webhook),
            )
            .await
    }

    /// GET `{scope}/webhooks`, one page.
    pub async fn list_webhooks(&self, page: Page) -> Result<WebhookList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/webhooks", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every webhook, draining pagination.
    pub async fn list_all_webhooks(&self) -> Result<Vec<Webhook>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_webhooks(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.webhooks);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/webhooks/{webhook_id}`.
    pub async fn get_webhook(&self, webhook_id: &str) -> Result<Webhook, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/webhooks/{}", self.base(), urlish(webhook_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/webhooks/{webhook_id}`, edit event types and inbox/pod
    /// targeting (see [`UpdateWebhook`]).
    pub async fn update_webhook(
        &self,
        webhook_id: &str,
        update: UpdateWebhook,
    ) -> Result<Webhook, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/webhooks/{}", self.base(), urlish(webhook_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `{scope}/webhooks/{webhook_id}`.
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/webhooks/{}", self.base(), urlish(webhook_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
