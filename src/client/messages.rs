use crate::{types::*, util::urlish, Error, Page};

impl super::Client {
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
            Some(serde_json::to_value(message).expect("serializable")),
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages (first page; see `list_messages_page`).
    pub async fn list_messages(&self, inbox_id: &str) -> Result<MessageList, Error> {
        self.list_messages_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/messages with pagination.
    pub async fn list_messages_page(
        &self,
        inbox_id: &str,
        page: Page,
    ) -> Result<MessageList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/messages", urlish(inbox_id)),
            &page.query(),
            None,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/messages/{message_id}
    pub async fn get_message(
        &self,
        inbox_id: &str,
        message_id: &str,
    ) -> Result<Message, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/messages/{}",
                urlish(inbox_id),
                urlish(message_id),
            ),
            &[],
            None,
        )
        .await
    }
}
