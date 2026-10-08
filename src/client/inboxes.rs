use crate::client::NoBody;
use crate::client::scope::{Inboxes, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Inboxes> Scoped<'_, S> {
    /// POST `{scope}/inboxes`, a new agent-owned email address. At [`Client::org`](crate::Client::org)
    /// this creates a standalone inbox; at [`Client::pod`](crate::Client::pod) it creates the inbox
    /// inside that pod. Free plans get `{username}@agentmail.to`; custom domains
    /// must be verified first.
    pub async fn create_inbox(&self, inbox: CreateInbox) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/inboxes", self.base()),
                &[],
                Some(&inbox),
            )
            .await
    }

    /// GET `{scope}/inboxes`, one page.
    pub async fn list_inboxes(&self, page: Page) -> Result<InboxList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/inboxes", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every inbox in the scope, draining pagination.
    pub async fn list_all_inboxes(&self) -> Result<Vec<Inbox>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_inboxes(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.inboxes);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/inboxes/{inbox_id}`.
    pub async fn get_inbox(&self, inbox_id: &str) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/inboxes/{inbox_id}`, update display name and/or metadata.
    pub async fn update_inbox(&self, inbox_id: &str, update: UpdateInbox) -> Result<Inbox, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `{scope}/inboxes/{inbox_id}`.
    pub async fn delete_inbox(&self, inbox_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/inboxes/{}", self.base(), urlish(inbox_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/inboxes/search`, full-text search over the scope's
    /// inboxes (display name, address, metadata).
    pub async fn search_inboxes(&self, q: &str, page: Page) -> Result<InboxList, Error> {
        let mut query = page.query();
        query.push(("q", q.to_string()));
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/inboxes/search", self.base()),
                &query,
                None::<&NoBody>,
            )
            .await
    }

    /// Every inbox matching `q`, draining pagination.
    pub async fn list_all_matching_inboxes(&self, q: &str) -> Result<Vec<Inbox>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .search_inboxes(
                    q,
                    Page {
                        limit: None,
                        page_token: token,
                    },
                )
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.inboxes);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }
}

impl Scoped<'_, crate::client::scope::InboxScope<'_>> {
    /// POST `/v0/inboxes/{inbox_id}/authorize`, complete the browser
    /// authorization flow for this inbox: the human approves in their browser,
    /// and this call exchanges the resulting `auth_token` for a scoped API
    /// key ([`AuthorizeInboxResult::api_key_id`]).
    pub async fn authorize(
        &self,
        authorize: AuthorizeInbox,
    ) -> Result<AuthorizeInboxResult, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("/v0/inboxes/{}/authorize", urlish(self.scope.0)),
                &[],
                Some(&authorize),
            )
            .await
    }
}
