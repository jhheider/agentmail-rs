# Changelog

All notable changes to this crate are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
API v0 upstream is pre-1.0; expect breaking releases while it settles.

## [0.4.1] - 2026-10-08

### Added

- The official SDKs' **WebSocket / realtime** event stream, behind the new
  off-by-default `websockets` feature: `Client::connect_realtime` opens the
  stream, sends the `subscribe` filter (event types, inboxes, pods), and
  returns a `RealtimeStream` whose `next_event` yields typed
  `RealtimeEvent`s for all 14 event types (message received/sent/delivered/
  opened/bounced/complained/rejected, domain verified, calendar
  created/updated/deleted/starting/ending/responded) plus `Subscribed`,
  `Other` (unknown frames survive as raw JSON), `Binary`, and `Closed`.
  Unknown event types decode as `Other` instead of failing, so additions
  don't break readers. No automatic reconnection, matching the SDKs.
  Dependencies (`tokio-tungstenite` on rustls + webpki roots, `futures-util`)
  are optional and only compile with the feature.
- The websocket host is derived from the API host by swapping the `api` DNS
  label for `ws` (`api.agentmail.to` -> `ws.agentmail.to`, EU:
  `ws.agentmail.eu`), overridable via `AGENTMAIL_WEBSOCKET_URL` (with
  `from_env`) or `Client::with_websocket_url`.

## [0.4.0] - 2026-10-07

Catches up with ~3 months of upstream drift (the API spec is now `1.0.0` and
the official SDKs ship near-daily): full REST surface parity again, plus a
weekly drift checker.

### Added

- **Calendars** (inbox scope): `get_calendar` / `update_calendar` (with
  `If-Match` etags), `get_agenda`, `list_calendar_events`,
  `create_calendar_event`, `get_calendar_event`, `update_calendar_event`
  (series `MutationMode`), `delete_calendar_event`, `list_calendar_event_instances`,
  `respond_to_calendar_event`, and the full `CalendarEvent` model
  (recurrence, attendees, duration modes, revisions).
- **Accounts** (human app connections): `list_accounts` /
  `get_account` / `update_account` at the org scope, plus account reads
  through inbox and pod handles.
- **Apps**: `list_apps`, `search_apps`, `get_app`, `list_app_accounts`,
  `connect_app` (magic-link connect flow).
- **Inbox authorization**: `inbox(id).authorize(AuthorizeInbox)` completes the
  browser flow and mints a scoped API key.
- **Agent**: `agent_attach_human` emails a claim link connecting a human.
- **Metrics**: `get_metrics_rates` (bounce/complaint rates) at all three
  scopes; `MetricsQuery::window`.
- **API keys**: `get_api_key` (org scope) and `update_api_key` (all scopes);
  the model now covers bearer and public-key credentials
  (`key_type`, `permissions`, `expires_at`, `status`, `public_key`, `created_by`).
- **Inbox search**: `search_inboxes` (+ `list_all_matching_inboxes`) at org
  and pod scopes.
- **Webhook custom headers**: `get_webhook_headers` / `update_webhook_headers`
  at all scopes, `headers` on `CreateWebhook`, `enabled` on `UpdateWebhook`.
- **Domains**: `get_domain_setup_link` (org scope), `inbound_enabled` /
  `tracking_enabled` / `reason`, `allow_conflicting_provider` on create;
  `VerificationRecord::reason`.
- `Message` gains the current spec fields: `reply_to`, `cc`, `bcc`, `headers`,
  `in_reply_to`, `references`, `size`, `created_at`, `updated_at`,
  `extracted_text`, `extracted_html`, `calendar_event_id`, `highlights`.
- `SendMessage` / `ReplyToMessage` gain `track_opens` (and reply gains `to`
  and `reply_all`); `Inbox` gains `status`, `metadata`, `updated_at` (create /
  update accept `status`); `Organization` gains billing/authentication ids;
  `ListEntry` gains `direction`, `list_type`, `pod_id`, `inbox_id`;
  `Draft` gains `preview`, `send_status`, `client_id`, `references`,
  `reply_to`; `UpdateDraft` gains `reply_to` and
  `add_attachments` / `remove_attachments`; `Attachment` gains `text_url`;
  `Webhook` gains `pod_id` / `inbox_id`.
- Weekly **drift checker**: `scripts/drift_check.py` diffs the live
  [OpenAPI spec](https://docs.agentmail.to/openapi.json) against the surface
  this crate binds; `.github/workflows/drift.yml` runs it Mondays and opens
  an issue on drift. `scripts/upstream_pin.json` pins the upstream commit the
  crate was built against.

### Changed (breaking)

- `AgentSignup::human_email` is now `Option<String>` (the API creates the
  inbox without a human when it is omitted; attach one later with
  `agent_attach_human`).
- `ApiKey::permissions` and `CreateApiKey::permissions` are now
  `Option<BTreeMap<String, bool>>` (were `serde_json::Value`) so permission
  additions don't need a client release.
- `Draft` no longer has `from` or `reply_all`: the spec's draft schema never
  returns them, so they never populated.
- `AgentVerifyResult` unchanged; `agent_verify` still posts `otp_code`.

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
