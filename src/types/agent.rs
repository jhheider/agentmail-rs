use serde::{Deserialize, Serialize};

/// Request body for `agent_sign_up`: start onboarding a new agent, which
/// emails a one-time code to `human_email` when given. `username` is the only
/// required field; without `human_email` the inbox is created unattached to a
/// human (attach one later with [`crate::Client::agent_attach_human`]).
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentSignup {
    /// The human's email, which receives the verification code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_email: Option<String>,
    /// The desired inbox username.
    pub username: String,
    /// Where the signup originated, optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Referrer, optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
}

/// The response to `agent_sign_up`: the new organization, inbox, and
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

/// The result of `agent_verify`.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentVerifyResult {
    /// Whether the one-time code was accepted.
    pub verified: bool,
}

/// Request body for `agent_attach_human`: email a claim link that connects a
/// human to this agent's organization.
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentAttachHuman {
    /// The human's email; they receive the claim link.
    pub human_email: String,
}

/// The response to `agent_attach_human`.
#[derive(Clone, Debug, Deserialize)]
pub struct AgentAttachHumanResult {
    /// The email the claim link was sent to.
    pub human_email: String,
    /// What to tell the human to do next.
    pub instructions: String,
}
