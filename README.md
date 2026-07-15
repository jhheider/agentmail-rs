# agentmail

[![crates.io](https://img.shields.io/crates/v/agentmail.svg)](https://crates.io/crates/agentmail)
[![docs.rs](https://img.shields.io/docsrs/agentmail)](https://docs.rs/agentmail)

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

Deliberately small: `reqwest` + `serde` + `thiserror`, with permissive
deserialization (unknown fields are ignored) so API additions don't break you.
TLS is **rustls with the ring provider** (no OpenSSL, no aws-lc-rs, no C
toolchain). The client installs ring as the process default at construction; if
your application already installs a crypto provider, that choice is respected.

## Install

```sh
cargo add agentmail
```

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

### Live smoke test

Creates, exercises, and lists real inboxes against the API:

```sh
AGENTMAIL_API_KEY=... cargo run --example smoke
```

## Scope

Not covered yet: threads, drafts, attachments, pods, domains, websockets.
PRs welcome.

## License

Licensed under either of [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE) at your option.
