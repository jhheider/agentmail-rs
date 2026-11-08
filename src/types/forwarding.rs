use serde::{Deserialize, Serialize};

/// Forwarding configuration for an org/pod/inbox.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ForwardingConfig {
    /// Addresses that receive forwarded mail.
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Addresses still awaiting verification.
    #[serde(default)]
    pub unverified: Vec<String>,
}

/// Request body for replacing forwarding addresses.
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateForwarding {
    /// Replacement address list (empty disables forwarding at this level).
    pub addresses: Vec<String>,
}

/// Request body for starting a verification.
#[derive(Clone, Debug, Serialize)]
pub struct CreateForwardingVerification {
    /// Email address to verify.
    pub email: String,
}

/// Response from starting a verification.
#[derive(Clone, Debug, Deserialize)]
pub struct ForwardingVerification {
    /// Email address being verified.
    pub email: String,
}

/// Request body for confirming a verification.
#[derive(Clone, Debug, Serialize)]
pub struct ConfirmForwardingVerification {
    /// Email address being verified.
    pub email: String,
    /// Confirmation code received by email.
    pub code: String,
}
