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

    /// GET `{scope}/webhooks/{webhook_id}/headers`, the names of the extra
    /// headers sent with each delivery (values are never returned).
    pub async fn get_webhook_headers(&self, webhook_id: &str) -> Result<WebhookHeaderNames, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/webhooks/{}/headers", self.base(), urlish(webhook_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/webhooks/{webhook_id}/headers`, set/replace headers and
    /// remove others by name. Responds 204 with no body.
    pub async fn update_webhook_headers(
        &self,
        webhook_id: &str,
        update: UpdateWebhookHeaders,
    ) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/webhooks/{}/headers", self.base(), urlish(webhook_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// POST `{scope}/webhooks/polling`, create a polling webhook: events are
    /// fetched by your client (over the realtime websocket feed or a polling
    /// endpoint) instead of pushed to a URL. The response carries the polling
    /// `token` exactly once; store it. `client_id` is the idempotency key:
    /// creating again with the same value returns the existing subscription
    /// and a fresh token.
    pub async fn create_polling_webhook(
        &self,
        webhook: CreatePollingWebhook,
    ) -> Result<CreatePollingWebhookResponse, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/webhooks/polling", self.base()),
                &[],
                Some(&webhook),
            )
            .await
    }

    /// GET `{scope}/webhooks/polling`, one page.
    pub async fn list_polling_webhooks(&self, page: Page) -> Result<PollingWebhookList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/webhooks/polling", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every polling webhook, draining pagination.
    pub async fn list_all_polling_webhooks(&self) -> Result<Vec<PollingWebhook>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_polling_webhooks(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.polling_webhooks);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/webhooks/polling/{webhook_id}`. The API also accepts your
    /// `client_id` in place of the `webhook_id`.
    pub async fn get_polling_webhook(&self, webhook_id: &str) -> Result<PollingWebhook, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/webhooks/polling/{}", self.base(), urlish(webhook_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// DELETE `{scope}/webhooks/polling/{webhook_id}`. The API also accepts
    /// your `client_id` in place of the `webhook_id`.
    pub async fn delete_polling_webhook(&self, webhook_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/webhooks/polling/{}", self.base(), urlish(webhook_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
