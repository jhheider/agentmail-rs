use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/inboxes/{inbox_id}/drafts -- create a draft. Supply
    /// `in_reply_to` to create a reply draft (with `reply_all` to address
    /// the whole thread) or `forward_of` to create a forward draft.
    pub async fn create_draft(&self, inbox_id: &str, draft: CreateDraft) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/drafts", urlish(inbox_id)),
            &[],
            Some(&draft),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts (first page; see
    /// [`Client::list_drafts_page`]).
    pub async fn list_drafts(&self, inbox_id: &str) -> Result<DraftList, Error> {
        self.list_drafts_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts with pagination. Feed
    /// [`DraftList::next_page_token`] back in as [`Page::page_token`]
    /// until it comes back `None`.
    pub async fn list_drafts_page(&self, inbox_id: &str, page: Page) -> Result<DraftList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/drafts", urlish(inbox_id)),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts/{draft_id}
    pub async fn get_draft(&self, inbox_id: &str, draft_id: &str) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}/drafts/{draft_id} -- edit fields on an
    /// existing draft. Passing `None` for an `Option` field leaves it
    /// unchanged; the API expects the field to be omitted, not set to null,
    /// when unchanged.
    pub async fn update_draft(
        &self,
        inbox_id: &str,
        draft_id: &str,
        update: UpdateDraft,
    ) -> Result<Draft, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            Some(&update),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}/drafts/{draft_id}
    pub async fn delete_draft(&self, inbox_id: &str, draft_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/drafts/{}",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/drafts/{draft_id}/send -- send a draft.
    /// The draft is deleted after a successful send.
    pub async fn send_draft(&self, inbox_id: &str, draft_id: &str) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/drafts/{}/send",
                urlish(inbox_id),
                urlish(draft_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
