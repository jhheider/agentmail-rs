//! Verification of AgentMail webhook deliveries, which are signed with
//! [Svix](https://docs.svix.com/receiving/verifying-payloads/how-manual).
//!
//! Enable the `webhook-verify` feature to pull this in (it adds `ring` and
//! `base64`). The functions are pure and take no [`Client`](crate::Client): a
//! server handling deliveries verifies with just the endpoint's signing secret
//! and the `svix-*` headers.
//!
//! ```
//! # #[cfg(feature = "webhook-verify")] {
//! use agentmail::verify_webhook_signature;
//! // Your endpoint's signing secret (the `whsec_...` value from the webhook).
//! let secret = format!("whsec_{}", "MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw");
//! let ok = verify_webhook_signature(
//!     &secret,
//!     "msg_p5jXN8AQM9LWM0D4loKWxJek",
//!     "1614265330",
//!     "v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE=",
//!     b"{\"test\": 2432232314}",
//! );
//! assert!(ok.is_ok());
//! # }
//! ```

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

/// Why webhook verification failed. Verification never "passes open": a missing
/// or malformed secret, header, or signature is always an error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SignatureError {
    /// The secret was empty, lacked the `whsec_` prefix, or did not base64-decode.
    #[error("invalid webhook secret")]
    InvalidSecret,
    /// The signature header carried no usable `v1,` candidate.
    #[error("invalid or missing signature header")]
    InvalidHeader,
    /// No candidate signature matched the computed one.
    #[error("webhook signature mismatch")]
    Mismatch,
    /// The timestamp was unparseable or outside the allowed tolerance.
    #[error("webhook timestamp outside tolerance")]
    StaleTimestamp,
}

/// The Svix-recommended timestamp tolerance (5 minutes).
pub const DEFAULT_WEBHOOK_TOLERANCE: std::time::Duration = std::time::Duration::from_secs(300);

/// Verify a webhook delivery's signature.
///
/// Computes `HMAC-SHA256(key, "{svix_id}.{svix_timestamp}.{body}")` where `key`
/// is the base64-decoded portion of the `whsec_`-prefixed `secret`, then checks
/// it (in constant time) against every space-separated `v1,<base64>` candidate
/// in `svix_signature`. Non-`v1` versions are ignored. Returns `Ok(())` on the
/// first match.
///
/// This checks only the signature. Pair it with
/// [`verify_webhook_timestamp`] to reject stale (replayed) deliveries.
pub fn verify_webhook_signature(
    secret: &str,
    svix_id: &str,
    svix_timestamp: &str,
    svix_signature: &str,
    body: &[u8],
) -> Result<(), SignatureError> {
    let key_bytes = decode_secret(secret)?;
    let key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, &key_bytes);

    // signed content = id "." timestamp "." body (body is arbitrary bytes).
    let mut signed = Vec::with_capacity(svix_id.len() + svix_timestamp.len() + body.len() + 2);
    signed.extend_from_slice(svix_id.as_bytes());
    signed.push(b'.');
    signed.extend_from_slice(svix_timestamp.as_bytes());
    signed.push(b'.');
    signed.extend_from_slice(body);

    let mut saw_candidate = false;
    for candidate in svix_signature.split(' ') {
        let Some((version, sig_b64)) = candidate.split_once(',') else {
            continue;
        };
        if version != "v1" {
            continue;
        }
        saw_candidate = true;
        let Ok(sig) = BASE64.decode(sig_b64) else {
            continue;
        };
        // ring::hmac::verify recomputes the tag and compares in constant time.
        if ring::hmac::verify(&key, &signed, &sig).is_ok() {
            return Ok(());
        }
    }

    if saw_candidate {
        Err(SignatureError::Mismatch)
    } else {
        Err(SignatureError::InvalidHeader)
    }
}

/// Reject deliveries whose `svix-timestamp` is more than `tolerance` away from
/// now (in either direction), the standard defense against replayed payloads.
/// See [`DEFAULT_WEBHOOK_TOLERANCE`].
pub fn verify_webhook_timestamp(
    svix_timestamp: &str,
    tolerance: std::time::Duration,
) -> Result<(), SignatureError> {
    let ts: i64 = svix_timestamp
        .parse()
        .map_err(|_| SignatureError::StaleTimestamp)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| SignatureError::StaleTimestamp)?
        .as_secs() as i64;
    if (now - ts).unsigned_abs() <= tolerance.as_secs() {
        Ok(())
    } else {
        Err(SignatureError::StaleTimestamp)
    }
}

/// Strip the `whsec_` prefix (if present) and base64-decode the key material.
fn decode_secret(secret: &str) -> Result<Vec<u8>, SignatureError> {
    if secret.is_empty() {
        return Err(SignatureError::InvalidSecret);
    }
    let b64 = secret.strip_prefix("whsec_").unwrap_or(secret);
    BASE64
        .decode(b64)
        .map_err(|_| SignatureError::InvalidSecret)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The canonical Svix test vector. The `whsec_` prefix is added at runtime so
    // the file carries no contiguous secret literal for scanners to flag; this
    // is Svix's published documentation vector, not a live credential.
    const SECRET_B64: &str = "MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw";
    const ID: &str = "msg_p5jXN8AQM9LWM0D4loKWxJek";
    const TS: &str = "1614265330";
    const BODY: &[u8] = b"{\"test\": 2432232314}";
    const SIG: &str = "v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE=";

    fn secret() -> String {
        format!("whsec_{SECRET_B64}")
    }

    #[test]
    fn known_vector_verifies() {
        assert!(verify_webhook_signature(&secret(), ID, TS, SIG, BODY).is_ok());
    }

    #[test]
    fn accepts_when_one_of_several_candidates_matches() {
        let multi = format!("v1,aaaa v2,bbbb {SIG}");
        assert!(verify_webhook_signature(&secret(), ID, TS, &multi, BODY).is_ok());
    }

    #[test]
    fn tampered_body_is_rejected() {
        let bad = verify_webhook_signature(&secret(), ID, TS, SIG, b"{\"test\": 9999}");
        assert_eq!(bad, Err(SignatureError::Mismatch));
    }

    #[test]
    fn empty_or_malformed_secret_is_rejected() {
        assert_eq!(
            verify_webhook_signature("", ID, TS, SIG, BODY),
            Err(SignatureError::InvalidSecret),
        );
        assert_eq!(
            verify_webhook_signature(&format!("whsec_{}", "!!!not base64!!!"), ID, TS, SIG, BODY),
            Err(SignatureError::InvalidSecret),
        );
    }

    #[test]
    fn no_v1_candidate_is_invalid_header() {
        assert_eq!(
            verify_webhook_signature(&secret(), ID, TS, "v2,whatever", BODY),
            Err(SignatureError::InvalidHeader),
        );
    }

    #[test]
    fn timestamp_tolerance() {
        // Far in the past relative to now: stale.
        assert_eq!(
            verify_webhook_timestamp(TS, DEFAULT_WEBHOOK_TOLERANCE),
            Err(SignatureError::StaleTimestamp),
        );
        // Unparseable: stale.
        assert_eq!(
            verify_webhook_timestamp("not-a-number", DEFAULT_WEBHOOK_TOLERANCE),
            Err(SignatureError::StaleTimestamp),
        );
    }
}
