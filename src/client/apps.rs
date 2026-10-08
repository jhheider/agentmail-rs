use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// GET /v0/apps, one page of the app directory.
    pub async fn list_apps(&self, filters: AppsListFilters) -> Result<AppList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/apps",
            &filters.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// Every app, draining pagination.
    pub async fn list_all_apps(&self) -> Result<Vec<App>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_apps(AppsListFilters {
                    limit: None,
                    page_token: token,
                    category: None,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.apps);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET /v0/apps/search, full-text search of the app directory (no
    /// pagination cursor).
    pub async fn search_apps(&self, q: &str, limit: Option<u32>) -> Result<AppSearchList, Error> {
        let mut query = Vec::new();
        query.push(("q", q.to_string()));
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        self.request(
            reqwest::Method::GET,
            "/v0/apps/search",
            &query,
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/apps/{app_id}.
    pub async fn get_app(&self, app_id: &str) -> Result<App, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/apps/{}", urlish(app_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/apps/{app_id}/accounts, one page of the humans connected to
    /// this app.
    pub async fn list_app_accounts(
        &self,
        app_id: &str,
        page: Page,
    ) -> Result<AppAccountList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/apps/{}/accounts", urlish(app_id)),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/apps/{app_id}/connect, start a human's connection to an app.
    /// The human opens [`ConnectAppResult::magic_url`] to approve before it
    /// expires.
    pub async fn connect_app(
        &self,
        app_id: &str,
        connect: ConnectApp,
    ) -> Result<ConnectAppResult, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/apps/{}/connect", urlish(app_id)),
            &[],
            Some(&connect),
        )
        .await
    }
}
