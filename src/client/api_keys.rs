use crate::client::NoBody;
use crate::client::scope::{ApiKeys, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: ApiKeys> Scoped<'_, S> {
    /// POST `{scope}/api-keys`, mint a new API key. The full secret is in
    /// [`CreatedApiKey::api_key`] and is shown only here; store it now.
    pub async fn create_api_key(&self, key: CreateApiKey) -> Result<CreatedApiKey, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/api-keys", self.base()),
                &[],
                Some(&key),
            )
            .await
    }

    /// GET `{scope}/api-keys`, one page.
    pub async fn list_api_keys(&self, page: Page) -> Result<ApiKeyList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/api-keys", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every API key, draining pagination.
    pub async fn list_all_api_keys(&self) -> Result<Vec<ApiKey>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_api_keys(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.api_keys);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// DELETE `{scope}/api-keys/{api_key_id}`.
    pub async fn delete_api_key(&self, api_key_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/api-keys/{}", self.base(), urlish(api_key_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/api-keys/{api_key_id}`, rename the key and/or replace
    /// its permission set.
    pub async fn update_api_key(
        &self,
        api_key_id: &str,
        update: UpdateApiKey,
    ) -> Result<ApiKey, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/api-keys/{}", self.base(), urlish(api_key_id)),
                &[],
                Some(&update),
            )
            .await
    }
}

/// Single-key reads exist only at the organization scope.
impl Scoped<'_, crate::client::scope::OrgScope> {
    /// GET `/v0/api-keys/{api_key_id}`. The inbox and pod scopes expose
    /// create/list/update/delete for their keys but no per-key read.
    pub async fn get_api_key(&self, api_key_id: &str) -> Result<ApiKey, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("/v0/api-keys/{}", urlish(api_key_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
