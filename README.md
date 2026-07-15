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
for agents. Coverage is the transactional core:

- **Inboxes**: create / list / get / delete (free plan: 3 inboxes,
  3k emails/month, `@agentmail.to` addresses)
- **Messages**: send / list / get
- **Webhooks**: create / list / delete (e.g. `message.received`)
- **Pagination** on every list call (`Page { limit, page_token }`)

Deliberately small: `reqwest` + `serde` + `thiserror`, with permissive
deserialization (unknown fields are ignored) so API additions don't break you.
Requests carry a 30-second default timeout. TLS is **rustls with the ring
provider** (no OpenSSL, no aws-lc-rs, no C toolchain). The client installs ring
as the process default at construction; if your application already installs a
crypto provider, that choice is respected.

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

## Parity roadmap

Where this crate stands against the official Python/TypeScript SDKs
(API v0). Covered now:

- [x] Inboxes: create / list / get / delete
- [x] Messages: send / list / get
- [x] Webhooks: create / list / delete
- [x] Pagination (`limit` / `page_token` cursors)

Not covered yet, roughly in the order we'd like to add them (PRs welcome):

- [ ] Threads (list / get)
- [ ] Attachments (send and fetch)
- [ ] Message updates (labels, read state) and reply-to threading
- [ ] Drafts (create / send)
- [ ] Message list filters (labels, before/after, from/to/subject)
- [ ] Webhook update / get; signature verification helper (Svix)
- [ ] Domains and pods
- [ ] WebSocket / real-time events
- [ ] Automatic retries with backoff

Changes land in the [changelog](CHANGELOG.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.
