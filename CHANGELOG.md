# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
API v0 upstream is pre-1.0; expect breaking releases while it settles.

## [0.1.0] - 2026-07-15

Initial release.

### Added

- `Client` with `new(key, base_url)` and `from_env()` (reads
  `AGENTMAIL_API_KEY`, optional `AGENTMAIL_BASE_URL`), a 30s default request
  timeout, and a `Debug` impl that redacts the API key.
- Inboxes: `create_inbox`, `list_inboxes`, `list_inboxes_page`, `get_inbox`,
  `delete_inbox`.
- Messages: `send_message`, `list_messages`, `list_messages_page`,
  `get_message`.
- Webhooks: `create_webhook`, `list_webhooks`, `list_webhooks_page`,
  `delete_webhook`.
- Pagination via `Page { limit, page_token }` on all list calls.
- Typed errors (`Error::MissingApiKey` / `Transport` / `Api` / `Decode`)
  and permissive deserialization (unknown fields ignored, optional fields
  default).
- TLS via rustls with the ring provider; no OpenSSL or C toolchain needed.
- Mock-server test suite (`tests/http.rs`) and a live smoke example
  (`examples/smoke.rs`).
