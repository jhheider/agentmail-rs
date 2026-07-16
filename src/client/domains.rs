use crate::client::NoBody;
use crate::client::scope::{Domains, Scoped};
use crate::{Error, Page, types::*, util::urlish};

impl<S: Domains> Scoped<'_, S> {
    /// POST `{scope}/domains`, add a sending domain. The response carries the
    /// DNS [`VerificationRecord`]s to publish.
    pub async fn create_domain(&self, domain: CreateDomain) -> Result<Domain, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/domains", self.base()),
                &[],
                Some(&domain),
            )
            .await
    }

    /// GET `{scope}/domains`, one page.
    pub async fn list_domains(&self, page: Page) -> Result<DomainList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/domains", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every domain, draining pagination.
    pub async fn list_all_domains(&self) -> Result<Vec<Domain>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_domains(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.domains);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{scope}/domains/{domain_id}`.
    pub async fn get_domain(&self, domain_id: &str) -> Result<Domain, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/domains/{}", self.base(), urlish(domain_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{scope}/domains/{domain_id}`, toggle feedback and subdomain
    /// settings.
    pub async fn update_domain(
        &self,
        domain_id: &str,
        update: UpdateDomain,
    ) -> Result<Domain, Error> {
        self.client
            .request(
                reqwest::Method::PATCH,
                &format!("{}/domains/{}", self.base(), urlish(domain_id)),
                &[],
                Some(&update),
            )
            .await
    }

    /// DELETE `{scope}/domains/{domain_id}`.
    pub async fn delete_domain(&self, domain_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/domains/{}", self.base(), urlish(domain_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// POST `{scope}/domains/{domain_id}/verify`, ask the API to re-check the
    /// DNS records; poll [`Scoped::get_domain`] for the resulting status.
    pub async fn verify_domain(&self, domain_id: &str) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/domains/{}/verify", self.base(), urlish(domain_id)),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/domains/{domain_id}/zone-file`, the DNS records as a
    /// ready-to-import zone file (plain text, not JSON).
    pub async fn get_domain_zone_file(&self, domain_id: &str) -> Result<String, Error> {
        self.client
            .request_text(
                reqwest::Method::GET,
                &format!("{}/domains/{}/zone-file", self.base(), urlish(domain_id)),
            )
            .await
    }
}
