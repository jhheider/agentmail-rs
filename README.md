# agentmail-rs

[![crates.io](https://img.shields.io/crates/v/agentmail-rs.svg)](https://crates.io/crates/agentmail-rs)
[![docs.rs](https://img.shields.io/docsrs/agentmail-rs)](https://docs.rs/agentmail-rs)

> **Unofficial.** This is a community Rust client, **not affiliated with or
> endorsed by AgentMail**. AgentMail ships official
> [Python](https://github.com/agentmail-to/agentmail-python) and
> [TypeScript](https://github.com/agentmail-to/agentmail-toolkit) SDKs; this
> crate fills the Rust gap. Wire shapes track AgentMail's public
> [OpenAPI spec](https://docs.agentmail.to/openapi.json) (API v0), which may
> change; pin a version and read the changelog.

A typed, `async` client for [AgentMail](https://agentmail.to), the email API
for agents, with **full coverage of the AgentMail API v0** at its canonical
scopes:

- **Inboxes**: create / list / get / update / delete
- **Threads**: list / filter / search / get / update / delete
- **Messages**: send / list / filter / search / get / update / delete, reply,
  reply-all, forward, raw source, batch get / update
- **Drafts**: create / list / get / update / delete / send
- **Attachments**: fetch metadata and download bytes (presigned)
- **Webhooks**: create / list / get / update / delete, plus optional Svix
  signature verification
- **Domains**: create / list / get / update / delete, verify, zone file
- **Pods**, **allow/block lists**, **metrics** (events + usage), **inbox
  events**, **API keys**, **organization**, and **agent** sign-up / verify
- **Pagination** on every list call (`Page { limit, page_token }`) and
  **automatic retries** with exponential backoff

Deliberately small: `reqwest` + `serde` + `thiserror` (plus `tokio` for retry
backoff), with permissive deserialization (unknown fields are ignored) so API
additions don't break you. Requests carry a 30-second default timeout. TLS is
**rustls with the ring provider** (no OpenSSL, no aws-lc-rs, no C toolchain).
The client installs ring as the process default at construction; if your
application already installs a crypto provider, that choice is respected.

### Features

- `retries` (default): automatic retries with backoff. Turn it off with
  `default-features = false` to drop the direct `tokio` dependency and make
  every request a single attempt; tune it with `Client::with_retry_policy`.
- `webhook-verify` (off by default): `verify_webhook_signature` for Svix-signed
  webhook deliveries. Adds `ring` (already the rustls provider) and `base64`.

## Install

```sh
cargo add agentmail-rs
```

The crate publishes as `agentmail-rs` (the bare `agentmail` name was taken) but
imports as `agentmail`.

## Usage

```rust,no_run
# async fn demo() -> Result<(), agentmail::Error> {
let client = agentmail::Client::from_env()?; // AGENTMAIL_API_KEY

let inbox = client
    .create_inbox(agentmail::CreateInbox {
        username: Some("my-agent".into()),
        ..Default::default()
    })
    .await?; // my-agent@agentmail.to

client
    .send_message(&inbox.inbox_id, agentmail::SendMessage {
        to: vec!["someone@example.com".into()],
        subject: Some("Hello".into()),
        text: Some("Sent from an agent's own inbox.".into()),
        ..Default::default()
    })
    .await?;

for m in client.list_messages(&inbox.inbox_id).await?.messages {
    println!("{:?}: {:?}", m.from, m.subject);
}
# Ok(()) }
```

`Client::from_env()` reads `AGENTMAIL_API_KEY` (and optional `AGENTMAIL_BASE_URL`
for the EU region or a mock server). For explicit config, use `Client::new(key,
base_url)`.

### Testing

The unit and mock-server tests (`cargo test`) run offline. For a live smoke
test that creates, exercises, and lists real inboxes against the API:

```sh
AGENTMAIL_API_KEY=... cargo run --example smoke
```

## Parity

This crate covers the full AgentMail API v0 surface that the official
Python/TypeScript SDKs expose, at the **canonical scope** for each resource
(inbox-scoped mail resources, top-level org resources). The scope-mirrored
variants the SDKs generate (`/pods/{id}/...` and `/inboxes/{id}/...` copies of
org-level resources) are not separately bound, since they return the same
shapes; open an issue if you need one.

Two things the official SDKs also lack and this crate treats as extras: there
is **no WebSocket / realtime** API to bind (AgentMail exposes none), and the
Svix **webhook signature verification** helper here goes slightly beyond the
SDKs (behind the `webhook-verify` feature).

Changes land in the [changelog](CHANGELOG.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.
