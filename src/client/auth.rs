use crate::client::NoBody;
use crate::{Client, Error, types::*};

impl Client {
    /// GET /v0/auth/me, the [`Identity`] the current API key authenticates as.
    pub async fn auth_me(&self) -> Result<Identity, Error> {
        self.request(reqwest::Method::GET, "/v0/auth/me", &[], None::<&NoBody>)
            .await
    }
}
