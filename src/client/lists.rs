use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// GET /v0/lists/{direction}/{kind} (first page; see
    /// [`Client::list_list_entries_page`]).
    pub async fn list_list_entries(
        &self,
        direction: ListDirection,
        kind: ListKind,
    ) -> Result<ListEntries, Error> {
        self.list_list_entries_page(direction, kind, Page::default())
            .await
    }

    /// GET /v0/lists/{direction}/{kind} with pagination. Feed
    /// [`ListEntries::next_page_token`] back in as [`Page::page_token`] until it
    /// comes back `None`.
    pub async fn list_list_entries_page(
        &self,
        direction: ListDirection,
        kind: ListKind,
        page: Page,
    ) -> Result<ListEntries, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/lists/{}/{}", direction.as_path(), kind.as_path()),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/lists/{direction}/{kind}, add an address or domain to the list.
    pub async fn create_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: CreateListEntry,
    ) -> Result<ListEntry, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/lists/{}/{}", direction.as_path(), kind.as_path()),
            &[],
            Some(&entry),
        )
        .await
    }

    /// GET /v0/lists/{direction}/{kind}/{entry}
    pub async fn get_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: &str,
    ) -> Result<ListEntry, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/lists/{}/{}/{}",
                direction.as_path(),
                kind.as_path(),
                urlish(entry),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// DELETE /v0/lists/{direction}/{kind}/{entry}
    pub async fn delete_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: &str,
    ) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/lists/{}/{}/{}",
                direction.as_path(),
                kind.as_path(),
                urlish(entry),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
