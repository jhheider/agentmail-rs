use crate::client::NoBody;
use crate::client::scope::{Lists, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Lists> Scoped<'_, S> {
    /// GET `{scope}/lists/{direction}/{kind}`, one page of entries.
    pub async fn list_entries(
        &self,
        direction: ListDirection,
        kind: ListKind,
        page: Page,
    ) -> Result<ListEntries, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/lists/{}/{}",
                    self.base(),
                    direction.as_path(),
                    kind.as_path()
                ),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every entry in the list, draining pagination.
    pub async fn list_all_entries(
        &self,
        direction: ListDirection,
        kind: ListKind,
    ) -> Result<Vec<ListEntry>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_entries(
                    direction,
                    kind,
                    Page {
                        limit: None,
                        page_token: token,
                    },
                )
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.entries);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// POST `{scope}/lists/{direction}/{kind}`, add an address or domain.
    pub async fn create_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: CreateListEntry,
    ) -> Result<ListEntry, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!(
                    "{}/lists/{}/{}",
                    self.base(),
                    direction.as_path(),
                    kind.as_path()
                ),
                &[],
                Some(&entry),
            )
            .await
    }

    /// GET `{scope}/lists/{direction}/{kind}/{entry}`.
    pub async fn get_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: &str,
    ) -> Result<ListEntry, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/lists/{}/{}/{}",
                    self.base(),
                    direction.as_path(),
                    kind.as_path(),
                    urlish(entry),
                ),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// DELETE `{scope}/lists/{direction}/{kind}/{entry}`.
    pub async fn delete_list_entry(
        &self,
        direction: ListDirection,
        kind: ListKind,
        entry: &str,
    ) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!(
                    "{}/lists/{}/{}/{}",
                    self.base(),
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
