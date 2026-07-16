use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/api-keys, mint a new API key. The full secret is in
    /// [`CreatedApiKey::api_key`] and is shown only here; store it now.
    pub async fn create_api_key(&self, key: CreateApiKey) -> Result<CreatedApiKey, Error> {
        self.request(reqwest::Method::POST, "/v0/api-keys", &[], Some(&key))
            .await
    }

    /// GET /v0/api-keys (first page; see [`Client::list_api_keys_page`]).
    pub async fn list_api_keys(&self) -> Result<ApiKeyList, Error> {
        self.list_api_keys_page(Page::default()).await
    }

    /// GET /v0/api-keys with pagination. Feed [`ApiKeyList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_api_keys_page(&self, page: Page) -> Result<ApiKeyList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/api-keys",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// DELETE /v0/api-keys/{api_key_id}
    pub async fn delete_api_key(&self, api_key_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/api-keys/{}", urlish(api_key_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
