use crate::{Error, Page, types::*, util::urlish};

impl super::Client {
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

    /// PATCH /v0/inboxes/{inbox_id}, update an inbox's display name or
    /// metadata. Only the fields you set are sent.
    pub async fn update_inbox(&self, inbox_id: &str, update: UpdateInbox) -> Result<Inbox, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!("/v0/inboxes/{}", urlish(inbox_id)),
            &[],
            Some(serde_json::to_value(update).expect("serializable")),
        )
        .await
    }
}
