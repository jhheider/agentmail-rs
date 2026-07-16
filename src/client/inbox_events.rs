use crate::client::NoBody;
use crate::client::scope::{InboxScope, Scoped};
use crate::{Error, Page, types::*};

impl Scoped<'_, InboxScope<'_>> {
    /// GET `/v0/inboxes/{inbox_id}/events`, one page of the inbox's audit log.
    pub async fn list_events(&self, page: Page) -> Result<InboxEventList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/events", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every inbox event, draining pagination.
    pub async fn list_all_events(&self) -> Result<Vec<InboxEvent>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_events(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.events);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }
}
