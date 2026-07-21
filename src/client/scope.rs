//! Type-level API scopes. Most AgentMail resources exist at more than one
//! scope (organization-wide, within a single inbox, or within a pod) at the
//! same URL shape under a different prefix. Rather than duplicate every method
//! per scope, each scope is a zero-cost marker type ([`OrgScope`], [`InboxScope`],
//! [`PodScope`]) and each resource is a capability trait ([`Threads`], [`Domains`],
//! ...) implemented only for the scopes that support it. The [`Scoped`] handle
//! carries the scope, and a resource's methods live in one generic
//! `impl<S: Capability> Scoped<'_, S>` block, so the compiler (not a runtime
//! 404) rejects `client.inbox(id).list_domains()` (inboxes have no domains).
//!
//! Get a handle from [`Client::org`](crate::Client::org), [`Client::inbox`](crate::Client::inbox), or [`Client::pod`](crate::Client::pod).

use crate::Client;
use crate::util::urlish;

/// The organization scope (top-level, `/v0/...`), spanning every inbox.
#[derive(Clone, Copy, Debug)]
pub struct OrgScope;

/// A single inbox scope (`/v0/inboxes/{inbox_id}/...`).
#[derive(Clone, Copy, Debug)]
pub struct InboxScope<'a>(pub &'a str);

/// A single pod scope (`/v0/pods/{pod_id}/...`).
#[derive(Clone, Copy, Debug)]
pub struct PodScope<'a>(pub &'a str);

/// A scope's URL prefix. Implemented by [`OrgScope`], [`InboxScope`], and [`PodScope`].
pub trait Scope {
    /// The path prefix for this scope (no trailing slash), e.g. `/v0` or
    /// `/v0/inboxes/ib_1`.
    fn base(&self) -> String;
}

impl Scope for OrgScope {
    fn base(&self) -> String {
        "/v0".to_string()
    }
}
impl Scope for InboxScope<'_> {
    fn base(&self) -> String {
        format!("/v0/inboxes/{}", urlish(self.0))
    }
}
impl Scope for PodScope<'_> {
    fn base(&self) -> String {
        format!("/v0/pods/{}", urlish(self.0))
    }
}

/// A [`Client`] bound to a [`Scope`]. Returned by [`Client::org`](crate::Client::org),
/// [`Client::inbox`](crate::Client::inbox), and [`Client::pod`](crate::Client::pod); the resource methods it exposes
/// depend on which capability traits the scope `S` implements.
#[derive(Clone, Copy, Debug)]
pub struct Scoped<'c, S> {
    pub(crate) client: &'c Client,
    pub(crate) scope: S,
}

impl<'c, S: Scope> Scoped<'c, S> {
    /// This scope's URL prefix.
    pub(crate) fn base(&self) -> String {
        self.scope.base()
    }
}

// ── Capability traits ─────────────────────────────────────────────────────────
// Each marks the scopes at which a resource exists. The methods themselves live
// in one generic `impl<S: Trait> Scoped<'_, S>` block in the resource's module.

/// Scopes with threads (org / inbox / pod).
pub trait Threads: Scope {}
impl Threads for OrgScope {}
impl Threads for InboxScope<'_> {}
impl Threads for PodScope<'_> {}

/// Scopes with readable drafts (org / inbox / pod). Draft *writes* (create,
/// update, delete, send) are inbox-only and live on `Scoped<InboxScope>`.
pub trait Drafts: Scope {}
impl Drafts for OrgScope {}
impl Drafts for InboxScope<'_> {}
impl Drafts for PodScope<'_> {}

/// Scopes with webhooks (org / inbox / pod).
pub trait Webhooks: Scope {}
impl Webhooks for OrgScope {}
impl Webhooks for InboxScope<'_> {}
impl Webhooks for PodScope<'_> {}

/// Scopes with allow/block lists (org / inbox / pod).
pub trait Lists: Scope {}
impl Lists for OrgScope {}
impl Lists for InboxScope<'_> {}
impl Lists for PodScope<'_> {}

/// Scopes with metrics (org / inbox / pod).
pub trait Metrics: Scope {}
impl Metrics for OrgScope {}
impl Metrics for InboxScope<'_> {}
impl Metrics for PodScope<'_> {}

/// Scopes with API keys (org / inbox / pod).
pub trait ApiKeys: Scope {}
impl ApiKeys for OrgScope {}
impl ApiKeys for InboxScope<'_> {}
impl ApiKeys for PodScope<'_> {}

/// Scopes with domains (org / pod). Inboxes have no domains.
pub trait Domains: Scope {}
impl Domains for OrgScope {}
impl Domains for PodScope<'_> {}

/// Scopes that contain inboxes (org / pod): list, create, get, update, delete
/// inboxes within the scope. A pod owns its inboxes; the org owns all of them.
pub trait Inboxes: Scope {}
impl Inboxes for OrgScope {}
impl Inboxes for PodScope<'_> {}

impl Client {
    /// The organization scope (top-level), spanning every inbox.
    pub fn org(&self) -> Scoped<'_, OrgScope> {
        Scoped {
            client: self,
            scope: OrgScope,
        }
    }

    /// Scope operations to a single inbox. Inbox-only resources (messages,
    /// events, draft writes) live here.
    ///
    /// The scope's capabilities are enforced at compile time: an inbox has no
    /// domains, so this does not compile:
    ///
    /// ```compile_fail
    /// # async fn f(client: agentmail::Client) -> Result<(), agentmail::Error> {
    /// client.inbox("ib_1").list_domains(Default::default()).await?;
    /// # Ok(()) }
    /// ```
    pub fn inbox<'a>(&'a self, inbox_id: &'a str) -> Scoped<'a, InboxScope<'a>> {
        Scoped {
            client: self,
            scope: InboxScope(inbox_id),
        }
    }

    /// Scope operations to a single pod.
    pub fn pod<'a>(&'a self, pod_id: &'a str) -> Scoped<'a, PodScope<'a>> {
        Scoped {
            client: self,
            scope: PodScope(pod_id),
        }
    }
}
