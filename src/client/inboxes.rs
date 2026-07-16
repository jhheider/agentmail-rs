use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/inboxes, a new agent-owned email address. Free plans get
    /// `{username}@agentmail.to`; custom domains must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.request(reqwest::Method::POST, "/v0/inboxes", &[], Some(&inbox))
            .await
    }

    /// GET /v0/inboxes (first page; see [`Client::list_inboxes_page`]).
    pub async fn list_inboxes(&self) -> Result<InboxList, Error> {
        self.list_inboxes_page(Page::default()).await
    }

    /// GET /v0/inboxes with pagination. Feed [`InboxList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_inboxes_page(&self, page: Page) -> Result<InboxList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/inboxes",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}, update the display name and/or metadata.
    pub async fn update_inbox(&self, inbox_id: &str, update: UpdateInbox) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            Some(&update),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
