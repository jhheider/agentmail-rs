use crate::{Client, Error, types::*};

impl Client {
    /// POST /v0/agent/sign-up, start onboarding a new agent. Emails a one-time
    /// code to `human_email`; confirm it with [`Client::agent_verify`]. Runs
    /// before any credential exists, so the client's bearer token (if any) is
    /// irrelevant.
    pub async fn agent_sign_up(&self, signup: AgentSignup) -> Result<AgentSignupResult, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/agent/sign-up",
            &[],
            Some(&signup),
        )
        .await
    }

    /// POST /v0/agent/verify, confirm the one-time code from
    /// [`Client::agent_sign_up`].
    pub async fn agent_verify(&self, otp_code: &str) -> Result<AgentVerifyResult, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/agent/verify",
            &[],
            Some(&serde_json::json!({ "otp_code": otp_code })),
        )
        .await
    }
}
