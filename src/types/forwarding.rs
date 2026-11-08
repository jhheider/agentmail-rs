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
    pub email: String,
}

/// Response from starting a verification.
#[derive(Clone, Debug, Deserialize)]
pub struct ForwardingVerification {
    pub email: String,
}

/// Request body for confirming a verification.
#[derive(Clone, Debug, Serialize)]
pub struct ConfirmForwardingVerification {
    pub email: String,
    pub code: String,
}
