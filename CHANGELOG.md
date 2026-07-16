# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
API v0 upstream is pre-1.0; expect breaking releases while it settles.

## [0.3.0] - 2026-07-16

Full three-scope coverage of the AgentMail API v0 (organization, inbox, pod),
reached through typed scope handles. **Breaking**: most methods moved from
`Client` onto a scope handle.

### Added

- Typed scope handles: `Client::org()`, `Client::inbox(id)`, `Client::pod(id)`
  return a `Scoped<S>` whose available methods are determined at compile time by
  the scope's capabilities (e.g. `client.inbox(id).list_domains(..)` does not
  compile: inboxes have no domains). Every previously inbox-only resource, plus
  the org- and pod-scoped variants of threads, webhooks, lists, domains,
  metrics, API keys, drafts, and inboxes, is now reachable. This completes 1:1
  surface parity with the official SDKs across all scopes.
- `list_all_*` drain helpers on every list endpoint (e.g.
  `inbox.list_all_messages`, `org.list_all_domains`) that fetch all pages into a
  `Vec`, with no new dependencies.

### Changed (breaking)

- Resource methods moved from `Client` onto scope handles:
  - Inbox-only (`client.inbox(id).*`): messages (send/reply/forward/raw/batch/
    ...), draft writes (create/update/delete/send), `list_events`.
  - Multi-scope (`client.org()/inbox(id)/pod(id).*`): threads, readable drafts,
    webhooks, allow/block lists, metrics, API keys; domains and inbox
    management on `org()`/`pod()`.
  - Account-global stays on `Client`: `list_pods`/`create_pod`/`get_pod`/
    `delete_pod`, `get_organization`, `auth_me`, `agent_sign_up`/`agent_verify`,
    `download_attachment`/`download_raw`.
- List calls take their filter/`Page` argument directly (no separate `_page` and
  `_filtered` variants): `list_messages(filters)`, `list_threads(filters)`,
  `list_domains(page)`, etc. `search_*` takes `(query, filters)`.
- `list_list_entries` is renamed `list_entries` (on the scope handle).
- `ListDirection` and `ListKind` are input-only path enums now (no `Deserialize`,
  no `Unknown` variant), so a call can't target a bogus list path.
- Renamed `list_inbox_events` to `inbox(id).list_events`.

## [0.2.1] - 2026-07-16

Polish release: no breaking changes.

### Added

- `Client::send_text(inbox_id, to, subject, text)`: the plain-text-to-one-
  recipient send in one line.
- `examples/webhook.rs`: end-to-end Svix webhook verification
  (`--features webhook-verify`).

### Changed

- docs.rs: feature-gated items (`RetryPolicy`, `with_retry_policy`, the
  `verify_*` helpers) now show an "available on feature" badge, and the
  crate-level docs surface a Coverage + Features overview.
- Resolved 44 `Client::` intra-doc links in the type modules that rendered as
  plain text instead of hyperlinks on docs.rs.
- README: MSRV (1.86) stated, `send_text` in the quickstart, webhook-example
  and Contributing sections.
- crates.io metadata: `async` keyword and `asynchronous` category for
  discoverability.

## [0.2.0] - 2026-07-16

Full coverage of the AgentMail API v0 surface exposed by the official SDKs, on
a modular internal structure. The public API stays flat (`agentmail::X`).

### Added

- Inboxes: `update_inbox`.
- Threads: `list_threads(_page)`, `list_threads_filtered`,
  `search_threads(_page)`, `get_thread`, `update_thread`, `delete_thread`.
- Messages: `reply_to_message`, `reply_all_to_message`, `forward_message`,
  `update_message`, `delete_message`, `list_messages_filtered`,
  `search_messages(_page)`, `get_raw_message` + `download_raw`,
  `batch_get_messages`, `batch_update_messages`.
- Drafts: `create_draft`, `list_drafts(_page)`, `get_draft`, `update_draft`,
  `delete_draft`, `send_draft`.
- Attachments: `get_message_attachment`, `get_thread_attachment`,
  `get_draft_attachment`, and `download_attachment` (presigned S3 fetch).
- Webhooks: `get_webhook`, `update_webhook`; optional `verify_webhook_signature`
  / `verify_webhook_timestamp` (Svix) behind the `webhook-verify` feature.
- Domains: `create_domain`, `list_domains(_page)`, `get_domain`,
  `update_domain`, `delete_domain`, `verify_domain`, `get_domain_zone_file`.
- Pods: `create_pod`, `list_pods(_page)`, `get_pod`, `delete_pod`.
- Lists: `list_list_entries(_page)`, `create_list_entry`, `get_list_entry`,
  `delete_list_entry` (allow/block, send/receive/reply).
- Metrics: `get_metrics_events`, `get_metrics_usage`; inbox audit log via
  `list_inbox_events(_page)`.
- API keys: `create_api_key`, `list_api_keys(_page)`, `delete_api_key`.
- Organization: `get_organization`. Auth: `auth_me` (`Identity`). Agent
  onboarding: `agent_sign_up`, `agent_verify`.
- `RetryPolicy` + `Client::with_retry_policy`: automatic retries with
  exponential backoff on timeout / 429 / 5xx, honoring `Retry-After`, behind
  the default-on `retries` feature (gates the direct `tokio` dependency).
- `Error::NoDownloadUrl` for attachment downloads with no presigned URL.

### Changed

- Internals split from a single `lib.rs` into `client/` and `types/` modules;
  no change to the public API surface.
- `Draft.attachments` and the new `Message.attachments` use the canonical
  `Attachment` type (full `AttachmentResponse` shape: adds `content_type`,
  `content_disposition`, `content_id`, `download_url`, `expires_at`).
- `update_message` now returns `UpdatedMessage` (`{ message_id, labels }`)
  rather than a full `Message`, matching the API response.

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
