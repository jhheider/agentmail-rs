use crate::client::NoBody;
use crate::client::scope::{Inboxes, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Inboxes> Scoped<'_, S> {
    /// POST `{scope}/inboxes`, a new agent-owned email address. At [`Client::org`](crate::Client::org)
    /// this creates a standalone inbox; at [`Client::pod`](crate::Client::pod) it creates the inbox
    /// inside that pod. Free plans get `{username}@agentmail.to`; custom domains
    /// must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/inboxes", self.base()),
                &[],
                Some(&inbox),
            )
            .await
    }

    /// GET `{scope}/inboxes`, one page.
    pub async fn list_inboxes(&self, page: Page) -> Result<InboxList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/inboxes", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every inbox in the scope, draining pagination.
    pub async fn list_all_inboxes(&self) -> Result<Vec<Inbox>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_inboxes(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.inboxes);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/inboxes/{inbox_id}`.
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/inboxes/{inbox_id}`, update display name and/or metadata.
    pub async fn update_inbox(&self, inbox_id: &str, update: UpdateInbox) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `{scope}/inboxes/{inbox_id}`.
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
