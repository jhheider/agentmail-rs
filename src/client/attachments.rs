use crate::client::NoBody;
use crate::{Client, Error, types::*, util::urlish};

impl Client {
    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}/attachments/{attachment_id}.
    /// The returned [`Attachment`] carries a short-lived `download_url`; pass it
    /// to [`Client::download_attachment`] to fetch the bytes.
    pub async fn get_message_attachment(
        &self,
        inbox_id: &str,
        message_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}/attachments/{}",
                urlish(inbox_id),
                urlish(message_id),
                urlish(attachment_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads/{thread_id}/attachments/{attachment_id}.
    pub async fn get_thread_attachment(
        &self,
        inbox_id: &str,
        thread_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/threads/{}/attachments/{}",
                urlish(inbox_id),
                urlish(thread_id),
                urlish(attachment_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/drafts/{draft_id}/attachments/{attachment_id}.
    pub async fn get_draft_attachment(
        &self,
        inbox_id: &str,
        draft_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/drafts/{}/attachments/{}",
                urlish(inbox_id),
                urlish(draft_id),
                urlish(attachment_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// Download an attachment's bytes from its presigned `download_url`. The URL
    /// is a short-lived S3 link fetched without the API bearer token, so obtain
    /// the [`Attachment`] from one of the `get_*_attachment` calls first (a
    /// plain response attachment carries no `download_url` and yields
    /// [`Error::NoDownloadUrl`]).
    pub async fn download_attachment(&self, attachment: &Attachment) -> Result<Vec<u8>, Error> {
        let url = attachment
            .download_url
            .as_deref()
            .ok_or(Error::NoDownloadUrl)?;
        // No bearer_auth: download_url is already an authenticated presigned URL.
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Api {
                status,
                body: String::from_utf8_lossy(&bytes).into_owned(),
            });
        }
        Ok(bytes.to_vec())
    }
}
