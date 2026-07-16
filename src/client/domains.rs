use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/domains, add a sending domain. The response carries the DNS
    /// [`VerificationRecord`]s to publish.
    pub async fn create_domain(&self, domain: CreateDomain) -> Result<Domain, Error> {
        self.request(reqwest::Method::POST, "/v0/domains", &[], Some(&domain))
            .await
    }

    /// GET /v0/domains (first page; see [`Client::list_domains_page`]).
    pub async fn list_domains(&self) -> Result<DomainList, Error> {
        self.list_domains_page(Page::default()).await
    }

    /// GET /v0/domains with pagination. Feed [`DomainList::next_page_token`]
    /// back in as [`Page::page_token`] until it comes back `None`.
    pub async fn list_domains_page(&self, page: Page) -> Result<DomainList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/domains",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/domains/{domain_id}
    pub async fn get_domain(&self, domain_id: &str) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// PATCH /v0/domains/{domain_id}, toggle feedback and subdomain settings.
    pub async fn update_domain(
        &self,
        domain_id: &str,
        update: UpdateDomain,
    ) -> Result<Domain, Error> {
        self.request(
            reqwest::Method::PATCH,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            Some(&update),
        )
        .await
    }

    /// DELETE /v0/domains/{domain_id}
    pub async fn delete_domain(&self, domain_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/domains/{}", urlish(domain_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/domains/{domain_id}/verify, ask the API to re-check the DNS
    /// records. Returns once the check is queued; poll [`Client::get_domain`]
    /// for the resulting status.
    pub async fn verify_domain(&self, domain_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::POST,
            &format!("/v0/domains/{}/verify", urlish(domain_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/domains/{domain_id}/zone-file, the DNS records as a ready-to-import
    /// zone file (plain text, not JSON).
    pub async fn get_domain_zone_file(&self, domain_id: &str) -> Result<String, Error> {
        self.request_text(
            reqwest::Method::GET,
            &format!("/v0/domains/{}/zone-file", urlish(domain_id)),
        )
        .await
    }
}
