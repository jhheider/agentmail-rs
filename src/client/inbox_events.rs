use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// GET /v0/inboxes/{inbox_id}/events (first page; see
    /// [`Client::list_inbox_events_page`]).
    pub async fn list_inbox_events(&self, inbox_id: &str) -> Result<InboxEventList, Error> {
        self.list_inbox_events_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/events with pagination, the inbox's audit log.
    pub async fn list_inbox_events_page(
        &self,
        inbox_id: &str,
        page: Page,
    ) -> Result<InboxEventList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/events", urlish(inbox_id)),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }
}
