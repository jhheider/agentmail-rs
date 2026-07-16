use crate::{types::*, util::urlish, Error, Page};

impl super::Client {
    // ── list_threads ─────────────────────────────────────────────────────────

    pub async fn list_threads(&self, inbox_id: &str) -> Result<ThreadList, Error> {
        self.list_threads_page(inbox_id, ThreadListFilters::default())
            .await
    }

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

    pub async fn search_threads(
        &self,
        inbox_id: &str,
        query: &str,
    ) -> Result<ThreadList, Error> {
        self.search_threads_page(inbox_id, query, ThreadListFilters::default())
            .await
    }

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

    pub async fn search_all_threads(&self, query: &str) -> Result<ThreadList, Error> {
        let body = serde_json::json!({ "q": query });
        self.request(reqwest::Method::POST, "/v0/threads/search", &[], Some(body))
            .await
    }

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
