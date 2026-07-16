//! Verify an incoming AgentMail webhook (Svix-signed). Needs the
//! `webhook-verify` feature:
//!
//!   cargo run --example webhook --features webhook-verify
//!
//! In a real server you'd read the `svix-id`, `svix-timestamp`, and
//! `svix-signature` headers and the raw request body off the HTTP request;
//! here we use AgentMail's published test vector so the example runs standalone.

use agentmail::{
    DEFAULT_WEBHOOK_TOLERANCE, SignatureError, verify_webhook_signature, verify_webhook_timestamp,
};

fn main() {
    // Your endpoint's signing secret (the `whsec_...` value from the webhook).
    let secret = format!("whsec_{}", "MfKQ9r8GKYqrTwjUPD8ILPZIo2LaLaSw");

    // These four come off the incoming request in production.
    let svix_id = "msg_p5jXN8AQM9LWM0D4loKWxJek";
    let svix_timestamp = "1614265330";
    let svix_signature = "v1,g0hM9SsE+OTPJTGt/tmIKtSyZlE3uFJELVlNIOLJ1OE=";
    let body = br#"{"test": 2432232314}"#;

    // 1) Confirm the payload was signed with your secret and not tampered with.
    match verify_webhook_signature(&secret, svix_id, svix_timestamp, svix_signature, body) {
        Ok(()) => println!("signature: OK"),
        Err(SignatureError::Mismatch) => {
            eprintln!("signature mismatch: reject the delivery");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("cannot verify: {e}");
            std::process::exit(1);
        }
    }

    // 2) Reject stale (replayed) deliveries. The test vector's timestamp is from
    //    2021, so this is expected to fail here; in production it guards replays.
    match verify_webhook_timestamp(svix_timestamp, DEFAULT_WEBHOOK_TOLERANCE) {
        Ok(()) => println!("timestamp: fresh"),
        Err(_) => println!("timestamp: stale (expected for the fixed test vector)"),
    }

    // Signature verified: now parse `body` as the event and act on it.
    println!("webhook: verified, ready to handle the event");
}
