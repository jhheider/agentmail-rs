use crate::client::NoBody;
use crate::{Client, Error, types::*};

impl Client {
    /// GET /v0/organizations, the organization the current key belongs to,
    /// including its resource counts and plan limits.
    pub async fn get_organization(&self) -> Result<Organization, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/organizations",
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/organizations/labs, the organization's AgentMail Labs
    /// enrollment. Labs features are offered in US production only; `enabled`
    /// says whether this organization has joined.
    pub async fn get_labs(&self) -> Result<OrganizationLabs, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/organizations/labs",
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// POST /v0/organizations/labs, join AgentMail Labs. Responds with the
    /// updated enrollment.
    pub async fn join_labs(&self) -> Result<OrganizationLabs, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/organizations/labs",
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
