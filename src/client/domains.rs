use crate::{types::*, util::urlish, Error, Page};

impl super::Client {
    // ── Domains ───────────────────────────────────────────────────────────────

    /// POST /v0/domains
    ///
    /// Register a custom domain for sending and receiving email.
    pub async fn create_domain(&self, domain: CreateDomain) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/domains",
            &[],
            Some(serde_json::to_value(domain).expect("serializable")),
        )
        .await
    }

    /// GET /v0/domains (first page; see `list_domains_page`).
    pub async fn list_domains(&self) -> Result<DomainList, Error> {
        self.list_domains_page(Page::default()).await
    }

    /// GET /v0/domains with pagination.
    pub async fn list_domains_page(&self, page: Page) -> Result<DomainList, Error> {
        self.request(reqwest::Method::GET, "/v0/domains", &page.query(), None)
            .await
    }

    /// GET /v0/domains/{domain_id}
    pub async fn get_domain(&self, domain_id: &str) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            None,
        )
        .await
    }

    /// PATCH /v0/domains/{domain_id}
    ///
    /// Update domain configuration (e.g. catch-all forwarding, DMARC policy).
    pub async fn update_domain(
        &self,
        domain_id: &str,
        domain: UpdateDomain,
    ) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            Some(serde_json::to_value(domain).expect("serializable")),
        )
        .await
    }

    /// DELETE /v0/domains/{domain_id}
    pub async fn delete_domain(&self, domain_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            None,
        )
        .await
    }

    /// POST /v0/domains/{domain_id}/verify
    ///
    /// Trigger domain ownership verification.
    pub async fn verify_domain(&self, domain_id: &str) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/domains/{}/verify", urlish(domain_id)),
            &[],
            None,
        )
        .await
    }

    /// GET /v0/domains/{domain_id}/zone-file
    ///
    /// Fetch the DNS zone file for the domain, returned as plain text.
    pub async fn get_domain_zone_file(&self, domain_id: &str) -> Result<String, Error> {
        let url = format!("{}/v0/domains/{}/zone-file", self.base_url, domain_id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(|e| Error::Http(e.into()))?;
        if !resp.status().is_success() {
            return Err(Error::Http(format!(
                "GET {} returned {}",
                url,
                resp.status()
            )));
        }
        resp.text()
            .await
            .map_err(|e| Error::Http(format!("failed to read zone file: {e}")))
    }
}
