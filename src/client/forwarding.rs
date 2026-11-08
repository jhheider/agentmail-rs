use crate::client::NoBody;
use crate::client::scope::{Forwarding, Scoped};
use crate::{Error, types::*, util::urlish};

impl<S: Forwarding> Scoped<'_, S> {
    /// GET `{scope}/forwarding` (org/pod/inbox).
    pub async fn get_forwarding(&self) -> Result<ForwardingConfig, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/forwarding", self.base()),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// PUT `{scope}/forwarding`, replace the forwarding addresses.
    pub async fn update_forwarding(
        &self,
        forwarding: UpdateForwarding,
    ) -> Result<ForwardingConfig, Error> {
        self.client
            .request(
                reqwest::Method::PUT,
                &format!("{}/forwarding", self.base()),
                &[],
                Some(&forwarding),
            )
            .await
    }

    /// DELETE `{scope}/forwarding`, remove forwarding.
    pub async fn delete_forwarding(&self) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::DELETE,
                &format!("{}/forwarding", self.base()),
                &[],
                None::<&NoBody>,
            )
            .await
    }

    /// POST `{scope}/forwarding/verifications`, email a confirmation code.
    pub async fn create_forwarding_verification(
        &self,
        req: CreateForwardingVerification,
    ) -> Result<ForwardingVerification, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/forwarding/verifications", self.base()),
                &[],
                Some(&req),
            )
            .await
    }

    /// POST `{scope}/forwarding/verifications/confirm`, verify the code.
    pub async fn confirm_forwarding_verification(
        &self,
        req: ConfirmForwardingVerification,
    ) -> Result<(), Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/forwarding/verifications/confirm", self.base()),
                &[],
                Some(&req),
            )
            .await
    }
}
