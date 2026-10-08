use crate::client::NoBody;
use crate::client::scope::Scoped;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// GET /v0/accounts, one page of human accounts (app connections).
    pub async fn list_accounts(&self, filters: AccountsListFilters) -> Result<AccountList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/accounts",
            &filters.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// Every account, draining pagination.
    pub async fn list_all_accounts(&self) -> Result<Vec<Account>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_accounts(AccountsListFilters {
                    limit: None,
                    page_token: token,
                    ascending: None,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.accounts);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET /v0/accounts/{account_id}.
    pub async fn get_account(&self, account_id: &str) -> Result<Account, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/accounts/{}", urlish(account_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// PATCH /v0/accounts/{account_id}/update, enable or disable the account.
    pub async fn update_account(
        &self,
        account_id: &str,
        update: UpdateAccount,
    ) -> Result<Account, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!("/v0/accounts/{}/update", urlish(account_id)),
            &[],
            Some(&update),
        )
        .await
    }
}

/// Accounts can also be listed through an inbox or pod handle; writes stay
/// org-only.
impl<S: private::AccountScopes> Scoped<'_, S> {
    /// GET `{scope}/accounts`, accounts connected through that inbox or pod.
    pub async fn list_accounts(&self, page: Page) -> Result<AccountList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/accounts", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/accounts/{account_id}`.
    pub async fn get_account(&self, account_id: &str) -> Result<Account, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/accounts/{}", self.base(), urlish(account_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}

mod private {
    use crate::client::scope::{InboxScope, PodScope, Scope};

    pub trait AccountScopes: Scope {}
    impl AccountScopes for InboxScope<'_> {}
    impl AccountScopes for PodScope<'_> {}
}
