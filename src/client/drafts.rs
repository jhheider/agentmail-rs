use crate::client::NoBody;
use crate::client::scope::{Drafts, InboxScope, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Drafts> Scoped<'_, S> {
    /// GET `{scope}/drafts`, one page. Drafts are readable at every scope but
    /// only created and sent from an [`Client::inbox`](crate::Client::inbox).
    pub async fn list_drafts(&self, page: Page) -> Result<DraftList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/drafts", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every draft, draining pagination.
    pub async fn list_all_drafts(&self) -> Result<Vec<Draft>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_drafts(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.drafts);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/drafts/{draft_id}`.
    pub async fn get_draft(&self, draft_id: &str) -> Result<Draft, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/drafts/{}", self.base(), urlish(draft_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/drafts/{draft_id}/attachments/{attachment_id}`.
    pub async fn get_draft_attachment(
        &self,
        draft_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/drafts/{}/attachments/{}",
                    self.base(),
                    urlish(draft_id),
                    urlish(attachment_id),
                ),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}

impl Scoped<'_, InboxScope<'_>> {
    /// POST `/v0/inboxes/{inbox_id}/drafts`, create a draft. Supply
    /// `in_reply_to` to create a reply draft (with `reply_all` to address the
    /// whole thread) or `forward_of` to create a forward draft.
    pub async fn create_draft(&self, draft: CreateDraft) -> Result<Draft, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/drafts", self.base()),
                &[],
                Some(&draft),
            )
            .await
    }

    /// PATCH `/v0/inboxes/{inbox_id}/drafts/{draft_id}`, edit an existing draft.
    pub async fn update_draft(&self, draft_id: &str, update: UpdateDraft) -> Result<Draft, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/drafts/{}", self.base(), urlish(draft_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `/v0/inboxes/{inbox_id}/drafts/{draft_id}`.
    pub async fn delete_draft(&self, draft_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/drafts/{}", self.base(), urlish(draft_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/drafts/{draft_id}/send`, send a draft. The
    /// draft is deleted after a successful send.
    pub async fn send_draft(&self, draft_id: &str) -> Result<SentMessage, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/drafts/{}/send", self.base(), urlish(draft_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
