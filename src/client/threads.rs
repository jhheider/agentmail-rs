use crate::{types::*, util::urlish, Error};

impl super::Client {
    // ── list_threads ─────────────────────────────────────────────────────────

    /// GET /v0/inboxes/{inbox_id}/threads (first page; see `list_threads_page`).
    pub async fn list_threads(&self, inbox_id: &str) -> Result<ThreadList, Error> {
        self.list_threads_page(inbox_id, ThreadListFilters::default())
            .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads with filtering + pagination.
    ///
    /// Feed [`ThreadList::next_page_token`] back in as
    /// [`ThreadListFilters::page_token`] until it comes back `None`.
    pub async fn list_threads_page(
        &self,
        inbox_id: &str,
        filters: ThreadListFilters,
    ) -> Result<ThreadList, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/inboxes/{}/threads", urlish(inbox_id)),
            &filters.query(),
            None,
        )
        .await
    }

    // ── search_threads (inbox-scoped) ────────────────────────────────────────

    /// GET /v0/inboxes/{inbox_id}/threads/search (first page).
    ///
    /// Full-text search across threads in a single inbox.
    pub async fn search_threads(
        &self,
        inbox_id: &str,
        query: &str,
    ) -> Result<ThreadList, Error> {
        self.search_threads_page(inbox_id, query, ThreadListFilters::default())
            .await
    }

    /// GET /v0/inboxes/{inbox_id}/threads/search with filtering + pagination.
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
            None,
        )
        .await
    }

    // ── get_thread ───────────────────────────────────────────────────────────

    /// GET /v0/inboxes/{inbox_id}/threads/{thread_id}
    ///
    /// Returns the full thread with all messages and attachments.
    pub async fn get_thread(
        &self,
        inbox_id: &str,
        thread_id: &str,
    ) -> Result<Thread, Error> {
        self.request(
            reqwest::Method::GET,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            None,
        )
        .await
    }

    // ── update_thread ────────────────────────────────────────────────────────

    /// PATCH /v0/inboxes/{inbox_id}/threads/{thread_id}
    ///
    /// Update labels on a thread. Only the labels you supply via
    /// [`UpdateThread::add_labels`] and [`UpdateThread::remove_labels`]
    /// are changed.
    pub async fn update_thread(
        &self,
        inbox_id: &str,
        thread_id: &str,
        update: UpdateThread,
    ) -> Result<Thread, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            Some(serde_json::to_value(update).expect("serializable")),
        )
        .await
    }

    // ── delete_thread ────────────────────────────────────────────────────────

    /// DELETE /v0/inboxes/{inbox_id}/threads/{thread_id}
    ///
    /// Permanently deletes a thread and all its messages.
    pub async fn delete_thread(&self, inbox_id: &str, thread_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!(
                "/v0/inboxes/{}/threads/{}",
                urlish(inbox_id),
                urlish(thread_id),
            ),
            &[],
            None,
        )
        .await
    }

    // ── search_all_threads (org-wide) ────────────────────────────────────────

    /// POST /v0/threads/search
    ///
    /// Full-text search across *all* inboxes in the organization (first page;
    /// see `search_all_threads_page`).
    pub async fn search_all_threads(&self, query: &str) -> Result<ThreadList, Error> {
        let body = serde_json::json!({ "q": query });
        self.request(reqwest::Method::POST, "/v0/threads/search", &[], Some(body))
            .await
    }

    /// POST /v0/threads/search with pagination.
    pub async fn search_all_threads_page(
        &self,
        query: &str,
        page: Page,
    ) -> Result<ThreadList, Error> {
        let body = serde_json::json!({ "q": query });
        self.request(
            reqwest::Method::POST,
            "/v0/threads/search",
            &page.query(),
            Some(body),
        )
        .await
    }
}
