use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/inboxes/{inbox_id}/messages/send
    pub async fn send_message(
        &self,
        inbox_id: &str,
        message: SendMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/messages/send", urlish(inbox_id)),
            &[],
            Some(&message),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages (first page; see
    /// [`Client::list_messages_page`]).
    pub async fn list_messages(&self, inbox_id: &str) -> Result<MessageList, Error> {
        self.list_messages_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/messages with pagination. Feed
    /// [`MessageList::next_page_token`] back in as [`Page::page_token`]
    /// until it comes back `None`.
    pub async fn list_messages_page(
        &self,
        inbox_id: &str,
        page: Page,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}
    pub async fn get_message(&self, inbox_id: &str, message_id: &str) -> Result<Message, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/{message_id}/reply -- reply to a
    /// message. The `to` field is derived from the parent message; only
    /// `text`/`html`/`cc`/`bcc` are caller-supplied.
    pub async fn reply_to_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/messages/{}/reply",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(&reply),
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/{message_id}/reply-all -- reply
    /// to all recipients of a message.
    pub async fn reply_all_to_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        reply: ReplyToMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/messages/{}/reply-all",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(&reply),
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}/messages/{message_id} -- update a
    /// message's labels (read state is a label, e.g. add/remove `unread`).
    /// Returns the message id and its labels after the change, not the full
    /// message body.
    pub async fn update_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        update: UpdateMessage,
    ) -> Result<UpdatedMessage, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(&update),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}/messages/{message_id} -- permanently
    /// deletes a message.
    pub async fn delete_message(&self, inbox_id: &str, message_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages with filters and pagination.
    /// Pass [`MessageListFilters`] with any filter fields set to narrow
    /// results. Feed [`MessageList::next_page_token`] back in as
    /// [`MessageListFilters::page_token`] until it comes back `None`.
    pub async fn list_messages_filtered(
        &self,
        inbox_id: &str,
        filters: MessageListFilters,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &filters.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/search with filters. Feed
    /// [`MessageList::next_page_token`] back in as
    /// [`MessageListFilters::page_token`] until it comes back `None`.
    pub async fn search_messages(&self, inbox_id: &str, query: &str) -> Result<MessageList, Error> {
        self.search_messages_page(inbox_id, query, MessageListFilters::default())
            .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/search with pagination.
    pub async fn search_messages_page(
        &self,
        inbox_id: &str,
        query: &str,
        filters: MessageListFilters,
    ) -> Result<MessageList, Error> {
        let mut q = filters.query();
        q.push(("q", query.to_string()));
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages/search", urlish(inbox_id)),
            &q,
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/{message_id}/forward, forward a
    /// message. Reuses [`SendMessage`] for the new recipients and any added
    /// body.
    pub async fn forward_message(
        &self,
        inbox_id: &str,
        message_id: &str,
        message: SendMessage,
    ) -> Result<SentMessage, Error> {
        self.request(
            reqwest::Method::POST,
            &format!(
                "/v0/inboxes/{}/messages/{}/forward",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            Some(&message),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}/raw, a presigned URL for
    /// the message's raw RFC 822 (`.eml`) source. Fetch the bytes with
    /// [`Client::download_raw`].
    pub async fn get_raw_message(
        &self,
        inbox_id: &str,
        message_id: &str,
    ) -> Result<RawMessage, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}/raw",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// Download the raw `.eml` bytes from a [`RawMessage`]'s presigned URL,
    /// without the API bearer token (the URL is already authenticated).
    pub async fn download_raw(&self, raw: &RawMessage) -> Result<Vec<u8>, Error> {
        let resp = self.http.get(&raw.download_url).send().await?;
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

    /// POST /v0/inboxes/{inbox_id}/messages/batch-get, fetch many messages by id
    /// in one call.
    pub async fn batch_get_messages(
        &self,
        inbox_id: &str,
        message_ids: Vec<String>,
    ) -> Result<BatchGetMessagesResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/messages/batch-get", urlish(inbox_id)),
            &[],
            Some(&BatchGetMessages { message_ids }),
        )
        .await
    }

    /// POST /v0/inboxes/{inbox_id}/messages/batch-update, apply the same label
    /// changes to many messages at once.
    pub async fn batch_update_messages(
        &self,
        inbox_id: &str,
        update: BatchUpdateMessages,
    ) -> Result<BatchUpdateMessagesResponse, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/inboxes/{}/messages/batch-update", urlish(inbox_id)),
            &[],
            Some(&update),
        )
        .await
    }
}
