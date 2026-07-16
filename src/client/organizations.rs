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
}
