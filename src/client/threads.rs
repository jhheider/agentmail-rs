use crate::client::NoBody;
use crate::client::scope::{Scoped, Threads};
use crate::{Error, types::*, util::urlish};

impl<S: Threads> Scoped<'_, S> {
    /// GET `{scope}/threads`, one page. Pass [`ThreadListFilters::default`] for
    /// the first unfiltered page; carry `next_page_token` back in for more, or
    /// use [`Scoped::list_all_threads`] to drain every page.
    pub async fn list_threads(&self, filters: ThreadListFilters) -> Result<ThreadList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/threads", self.base()),
                &filters.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every thread matching `filters`, draining pagination.
    pub async fn list_all_threads(
        &self,
        mut filters: ThreadListFilters,
    ) -> Result<Vec<Thread>, Error> {
        let mut out = Vec::new();
        loop {
            let page = self.list_threads(filters.clone()).await?;
            let next = page.next_page_token;
            out.extend(page.threads);
            match next {
                Some(token) => filters.page_token = Some(token),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/threads/search`, full-text search (`q` required); matches
    /// come back in [`Thread::highlights`].
    pub async fn search_threads(
        &self,
        query: &str,
        filters: ThreadListFilters,
    ) -> Result<ThreadList, Error> {
        let mut q = filters.query();
        q.push(("q", query.to_string()));
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/threads/search", self.base()),
                &q,
                None::<&NoBody>,
            )
            .await
    }

    /// Every thread matching a search, draining pagination.
    pub async fn search_all_threads(
        &self,
        query: &str,
        mut filters: ThreadListFilters,
    ) -> Result<Vec<Thread>, Error> {
        let mut out = Vec::new();
        loop {
            let page = self.search_threads(query, filters.clone()).await?;
            let next = page.next_page_token;
            out.extend(page.threads);
            match next {
                Some(token) => filters.page_token = Some(token),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/threads/{thread_id}`, the full thread with its messages.
    pub async fn get_thread(&self, thread_id: &str) -> Result<Thread, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/threads/{}", self.base(), urlish(thread_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/threads/{thread_id}`, add and/or remove labels across the
    /// thread. Returns the thread id and its labels after the change.
    pub async fn update_thread(
        &self,
        thread_id: &str,
        update: UpdateThread,
    ) -> Result<UpdatedThread, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/threads/{}", self.base(), urlish(thread_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `{scope}/threads/{thread_id}`.
    pub async fn delete_thread(&self, thread_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/threads/{}", self.base(), urlish(thread_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/threads/{thread_id}/attachments/{attachment_id}`, the
    /// attachment metadata with a short-lived `download_url`; fetch the bytes
    /// with [`Client::download_attachment`](crate::Client::download_attachment).
    pub async fn get_thread_attachment(
        &self,
        thread_id: &str,
        attachment_id: &str,
    ) -> Result<Attachment, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/threads/{}/attachments/{}",
                    self.base(),
                    urlish(thread_id),
                    urlish(attachment_id),
                ),
                &[],
                None::<&NoBody>,
            )
            .await
    }
}
