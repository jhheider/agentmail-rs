use serde::{Deserialize, Serialize};

/// Request body for [`Client::agent_sign_up`]: start onboarding a new agent,
/// which emails a one-time code to `human_email`.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentSignup {
    /// The human's email, which receives the verification code.
    pub human_email: String,
    /// The desired inbox username.
    pub username: String,
    /// Where the signup originated, optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Referrer, optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
}

/// The response to [`Client::agent_sign_up`]: the new organization, inbox, and
/// its API key.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentSignupResult {
    /// The new organization id.
    pub organization_id: String,
    /// The new inbox id.
    pub inbox_id: String,
    /// The new inbox's API key.
    pub api_key: String,
}

/// The result of [`Client::agent_verify`].
#[derive(Clone, Debug, Deserialize)]
pub struct AgentVerifyResult {
    /// Whether the one-time code was accepted.
    pub verified: bool,
}
