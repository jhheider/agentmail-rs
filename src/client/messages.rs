use crate::client::NoBody;
use crate::client::scope::{InboxScope, Scoped};
use crate::{Error, types::*, util::urlish};

/// Messages exist only within an inbox, so these methods live on
/// `client.inbox(id)`.
impl Scoped<'_, InboxScope<'_>> {
    /// POST `/v0/inboxes/{inbox_id}/messages/send`.
    pub async fn send_message(&self, message: SendMessage) -> Result<SentMessage, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/send", self.base()),
                &[],
                Some(&message),
            )
            .await
    }

    /// Send a plain-text message to a single recipient: the common case, in one
    /// line. For anything richer (HTML, cc/bcc, attachments, labels), build a
    /// [`SendMessage`] and call [`Scoped::send_message`].
    pub async fn send_text(
        &self,
        to: &str,
        subject: &str,
        text: &str,
    ) -> Result<SentMessage, Error> {
        self.send_message(SendMessage {
            to: vec![to.to_string()],
            subject: Some(subject.to_string()),
            text: Some(text.to_string()),
            ..Default::default()
        })
        .await
    }

    /// GET `/v0/inboxes/{inbox_id}/messages`, one page. Pass
    /// [`MessageListFilters::default`] for the first unfiltered page.
    pub async fn list_messages(&self, filters: MessageListFilters) -> Result<MessageList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/messages", self.base()),
                &filters.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every message matching `filters`, draining pagination.
    pub async fn list_all_messages(
        &self,
        mut filters: MessageListFilters,
    ) -> Result<Vec<Message>, Error> {
        let mut out = Vec::new();
        loop {
            let page = self.list_messages(filters.clone()).await?;
            let next = page.next_page_token;
            out.extend(page.messages);
            match next {
                Some(token) => filters.page_token = Some(token),
                None => return Ok(out),
            }
        }
    }

    /// GET `/v0/inboxes/{inbox_id}/messages/search` (`q` required).
    pub async fn search_messages(
        &self,
        query: &str,
        filters: MessageListFilters,
    ) -> Result<MessageList, Error> {
        let mut q = filters.query();
        q.push(("q", query.to_string()));
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/messages/search", self.base()),
                &q,
                None::<&NoBody>,
            )
            .await
    }

    /// Every message matching a search, draining pagination.
    pub async fn search_all_messages(
        &self,
        query: &str,
        mut filters: MessageListFilters,
    ) -> Result<Vec<Message>, Error> {
        let mut out = Vec::new();
        loop {
            let page = self.search_messages(query, filters.clone()).await?;
            let next = page.next_page_token;
            out.extend(page.messages);
            match next {
                Some(token) => filters.page_token = Some(token),
                None => return Ok(out),
            }
        }
    }

    /// GET `/v0/inboxes/{inbox_id}/messages/{message_id}`.
    pub async fn get_message(&self, message_id: &str) -> Result<Message, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/messages/{}", self.base(), urlish(message_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `/v0/inboxes/{inbox_id}/messages/{message_id}`, add/remove labels
    /// (read state is a label). Returns the id and labels after the change.
    pub async fn update_message(
        &self,
        message_id: &str,
        update: UpdateMessage,
    ) -> Result<UpdatedMessage, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/messages/{}", self.base(), urlish(message_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `/v0/inboxes/{inbox_id}/messages/{message_id}`.
    pub async fn delete_message(&self, message_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/messages/{}", self.base(), urlish(message_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/messages/{message_id}/reply`, reply to a
    /// message (recipients derived from the parent).
    pub async fn reply_to_message(
        &self,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/{}/reply", self.base(), urlish(message_id)),
                &[],
                Some(&reply),
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/messages/{message_id}/reply-all`.
    pub async fn reply_all_to_message(
        &self,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/{}/reply-all", self.base(), urlish(message_id)),
                &[],
                Some(&reply),
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/messages/{message_id}/forward`. Reuses
    /// [`SendMessage`] for the new recipients and any added body.
    pub async fn forward_message(
        &self,
        message_id: &str,
        message: SendMessage,
    ) -> Result<SentMessage, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/{}/forward", self.base(), urlish(message_id)),
                &[],
                Some(&message),
            )
            .await
    }

    /// GET `/v0/inboxes/{inbox_id}/messages/{message_id}/raw`, a presigned URL
    /// for the raw RFC 822 (`.eml`) source; fetch the bytes with
    /// [`Client::download_raw`](crate::Client::download_raw).
    pub async fn get_raw_message(&self, message_id: &str) -> Result<RawMessage, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/messages/{}/raw", self.base(), urlish(message_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// GET `/v0/inboxes/{inbox_id}/messages/{message_id}/attachments/{attachment_id}`.
    pub async fn get_message_attachment(
        &self,
        message_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/messages/{}/attachments/{}",
                    self.base(),
                    urlish(message_id),
                    urlish(attachment_id),
                ),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/messages/batch-get`, fetch many messages by
    /// id in one call.
    pub async fn batch_get_messages(
        &self,
        message_ids: Vec<String>,
    ) -> Result<BatchGetMessagesResponse, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/batch-get", self.base()),
                &[],
                Some(&BatchGetMessages { message_ids }),
            )
            .await
    }

    /// POST `/v0/inboxes/{inbox_id}/messages/batch-update`, apply the same label
    /// changes to many messages at once.
    pub async fn batch_update_messages(
        &self,
        update: BatchUpdateMessages,
    ) -> Result<BatchUpdateMessagesResponse, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/messages/batch-update", self.base()),
                &[],
                Some(&update),
            )
            .await
    }
}
