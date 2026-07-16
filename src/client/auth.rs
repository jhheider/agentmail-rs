use crate::{Error, types::*};

impl super::Client {
    /// GET /v0/auth/me, returns the current identity associated with the
    /// API key. Useful for scoped keys (pod_key, inbox_key) to discover
    /// their parent organization, pod, or inbox at runtime.
    ///
    /// ```no_run
    /// # async fn demo() -> Result<(), agentmail::Error> {
    /// let client = agentmail::Client::from_env()?;
    /// let identity = client.auth_me().await?;
    /// println!("org: {}", identity.organization_id);
    /// # Ok(()) }
    /// ```
    pub async fn auth_me(&self) -> Result<Identity, Error> {
        self.request(reqwest::Method::GET, "/v0/auth/me", &[], None)
            .await
    }
}
