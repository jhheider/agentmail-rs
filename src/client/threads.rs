use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// GET /v0/inboxes/{inbox_id}/threads (first page; see
    /// [`Client::list_threads_page`]).
    pub async fn list_threads(&self, inbox_id: &str) -> Result<ThreadList, Error> {
        self.list_threads_page(inbox_id, Page::default()).await
    }

    /// GET /v0/inboxes/{inbox_id}/threads with pagination. Feed
    /// [`ThreadList::next_page_token`] back in as [`Page::page_token`] until it
    /// comes back `None`.
    pub async fn list_threads_page(&self, inbox_id: &str, page: Page) -> Result<ThreadList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/threads", urlish(inbox_id)),
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads with filters and pagination. Pass
    /// [`ThreadListFilters`] with any fields set to narrow results.
    pub async fn list_threads_filtered(
        &self,
        inbox_id: &str,
        filters: ThreadListFilters,
    ) -> Result<ThreadList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/threads", urlish(inbox_id)),
            &filters.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads/search, full-text search over the
    /// inbox's threads. The matched fragments come back in
    /// [`Thread::highlights`].
    pub async fn search_threads(&self, inbox_id: &str, query: &str) -> Result<ThreadList, Error> {
        self.search_threads_page(inbox_id, query, ThreadListFilters::default())
            .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads/search with filters and pagination.
    pub async fn search_threads_page(
        &self,
        inbox_id: &str,
        query: &str,
        filters: ThreadListFilters,
    ) -> Result<ThreadList, Error> {
        let mut q = filters.query();
        q.push(("q", query.to_string()));
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/threads/search", urlish(inbox_id)),
            &q,
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads/{thread_id}, the full thread with its
    /// messages.
    pub async fn get_thread(&self, inbox_id: &str, thread_id: &str) -> Result<Thread, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// PATCH /v0/inboxes/{inbox_id}/threads/{thread_id}, add and/or remove
    /// labels across the thread. Returns the thread id and its labels after the
    /// change.
    pub async fn update_thread(
        &self,
        inbox_id: &str,
        thread_id: &str,
        update: UpdateThread,
    ) -> Result<UpdatedThread, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            Some(&update),
        )
        .await
    }

    /// DELETE /v0/inboxes/{inbox_id}/threads/{thread_id}.
    pub async fn delete_thread(&self, inbox_id: &str, thread_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
