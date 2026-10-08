use serde::{Deserialize, Serialize};

use crate::util::QueryBuilder;

/// An inbox's calendar, from `get_calendar` / `update_calendar`.
#[derive(Clone, Debug, Deserialize)]
pub struct Calendar {
    /// The inbox the calendar belongs to.
    pub inbox_id: String,
    /// IANA timezone the calendar renders in, e.g. `America/New_York`.
    pub timezone: String,
    /// Number of events on the calendar.
    pub event_count: u64,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
    /// Last-update timestamp (RFC 3339).
    pub updated_at: String,
    /// Opaque revision token; pass it as `etag` on writes to reject
    /// concurrent modifications (428 otherwise).
    pub etag: String,
}

/// Whether an event occurs once or as part of a series.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    /// A one-off event.
    Single,
    /// The defining event of a recurring series.
    Series,
    /// One materialized occurrence of a series.
    Instance,
    /// A kind this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Confirmation state of an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    /// The event is on as scheduled.
    Confirmed,
    /// The event is provisional.
    Tentative,
    /// The event was cancelled.
    Cancelled,
    /// A status this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// How an event's `duration_value` is interpreted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DurationMode {
    /// `duration_value` counts nominal calendar units (a `P1D` day stays a day
    /// across DST shifts).
    Nominal,
    /// `duration_value` counts exact seconds.
    Exact,
    /// A mode this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// Where an event came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    /// Created through the API.
    Api,
    /// Parsed out of an emailed invite.
    Email,
    /// A source this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// The organizer's response state on an event the inbox was invited to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    /// No reply yet.
    NeedsAction,
    /// Going.
    Accepted,
    /// Not going.
    Declined,
    /// Maybe.
    Tentative,
    /// A status this client version does not recognize.
    #[serde(other)]
    Unknown,
}

/// An attendee on an event.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CalendarAttendee {
    /// The attendee's email.
    pub email: String,
    /// Display name, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The attendee's reply state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// `required` or `optional`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The attendee's reply comment, when they left one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// When the attendee last replied (RFC 3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub responded_at: Option<String>,
}

/// Recurrence rule and exceptions on an event. `rdates` entries are either an
/// RFC 3339 timestamp or a `{start, end?, duration?}` object, so they are kept
/// as raw JSON.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Recurrence {
    /// RFC 5545 RRULE, e.g. `FREQ=WEEKLY;BYDAY=MO,WE`.
    pub rule: String,
    /// Occurrence start timestamps to exclude (RFC 3339).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exdates: Vec<String>,
    /// Extra occurrences to add: RFC 3339 strings or `{start, end?, duration?}`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rdates: Vec<serde_json::Value>,
    /// Cutoff after which the series stops recurring (RFC 3339).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncate_before: Option<String>,
}

/// A calendar event, as the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct CalendarEvent {
    /// Unique event id.
    pub event_id: String,
    /// Whether this is a single event, a series definition, or an instance.
    #[serde(default)]
    pub kind: Option<EventKind>,
    /// The series this instance belongs to, when applicable.
    #[serde(default)]
    pub series_id: Option<String>,
    /// The series' start this instance shifted from, when moved (RFC 3339).
    #[serde(default)]
    pub original_start: Option<String>,
    /// Same as [`CalendarEvent::original_start`], on `instance` events.
    #[serde(default)]
    pub original_start_at: Option<String>,
    /// Whether this instance deviates from the series.
    #[serde(default)]
    pub is_exception: Option<bool>,
    /// Event title.
    pub title: String,
    /// Long description.
    #[serde(default)]
    pub description: Option<String>,
    /// Location, when set.
    #[serde(default)]
    pub location: Option<String>,
    /// Your own JSON stored alongside the event.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Confirmation state.
    #[serde(default)]
    pub status: Option<EventStatus>,
    /// Whether the event lasts all day (then `start`/`end` are dates).
    pub all_day: bool,
    /// Display start (RFC 3339, or a date for all-day events).
    pub start: String,
    /// Display end.
    pub end: String,
    /// IANA timezone the event renders in.
    pub timezone: String,
    /// How `duration_value` is interpreted.
    #[serde(default)]
    pub duration_mode: Option<DurationMode>,
    /// Length in units picked by `duration_mode`.
    pub duration_value: i64,
    /// Absolute start (RFC 3339).
    pub start_at: String,
    /// Absolute end (RFC 3339).
    pub end_at: String,
    /// Recurrence rule, when the event repeats.
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    /// Invitees and their replies.
    #[serde(default)]
    pub attendees: Vec<CalendarAttendee>,
    /// Number of attendees.
    pub attendee_count: u32,
    /// The inbox's own reply state, when it was invited.
    #[serde(default)]
    pub response_status: Option<ResponseStatus>,
    /// iCalendar UID, stable across the series.
    pub uid: String,
    /// iCalendar sequence number, bumped on material changes.
    pub sequence: u32,
    /// Whether the event came from the API or an email.
    #[serde(default)]
    pub source: Option<EventSource>,
    /// Organizer address, when known.
    #[serde(default)]
    pub organizer_email: Option<String>,
    /// The email message the invite arrived in, when from email.
    #[serde(default)]
    pub origin_message_id: Option<String>,
    /// Monotonic revision, for optimistic concurrency.
    #[serde(default)]
    pub resource_revision: Option<u64>,
    /// Opaque revision token for `If-Match` writes, when returned.
    #[serde(default)]
    pub etag: Option<String>,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
    /// Last-update timestamp (RFC 3339).
    pub updated_at: String,
}

/// Read-staleness control for calendar reads. `primary` (the API default)
/// reads the authoritative store; `eventual` may serve a replica.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consistency {
    /// May read a replica; cheaper and usually fine.
    Eventual,
    /// Always read the primary store.
    Primary,
}

impl Consistency {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Consistency::Eventual => "eventual",
            Consistency::Primary => "primary",
        }
    }
}

/// Which occurrences a series mutation applies to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationMode {
    /// Only the addressed instance.
    Single,
    /// The addressed instance and every later one.
    Future,
}

impl MutationMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            MutationMode::Single => "single",
            MutationMode::Future => "future",
        }
    }
}

/// Windowing and pagination for `get_agenda` and `list_event_instances`.
#[derive(Clone, Debug, Default)]
pub struct CalendarEventsQuery {
    /// Only occurrences starting at or after this instant (RFC 3339).
    pub after: Option<String>,
    /// Only occurrences starting before this instant (RFC 3339).
    pub before: Option<String>,
    /// Also include occurrences overlapping the window.
    pub include_overlapping: Option<bool>,
    /// Maximum occurrences per page.
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
    /// Read-staleness control.
    pub consistency: Option<Consistency>,
}

impl CalendarEventsQuery {
    pub(crate) fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("after", self.after.as_ref())
            .opt("before", self.before.as_ref())
            .opt("include_overlapping", self.include_overlapping.as_ref())
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .opt(
                "consistency",
                self.consistency.map(Consistency::as_str).as_ref(),
            )
            .build()
    }
}

/// Body for `create_calendar_event`. `title`, `start`, and `end` are required
/// by the API.
#[derive(Clone, Debug, Default, Serialize)]
pub struct CreateCalendarEvent {
    /// Your own idempotency/reference id for the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Event title.
    pub title: String,
    /// Long description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Your own JSON stored alongside the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Confirmation state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>,
    /// All-day event (`start`/`end` become dates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_day: Option<bool>,
    /// Display start (RFC 3339, or a date for all-day events).
    pub start: String,
    /// Display end.
    pub end: String,
    /// IANA timezone; defaults to the calendar's.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// How to interpret `duration_value` (defaults to the API's).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_mode: Option<DurationMode>,
    /// Recurrence rule, to make the event a series.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<Recurrence>,
    /// Invitees.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<CalendarAttendee>,
    /// Email invites to the attendees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_invites: Option<bool>,
}

/// Body for `update_calendar_event`. Fields left `None` stay unchanged; there
/// is no way to set a field back to null from this client (the API accepts
/// nulls for `description`/`location`/`metadata`/`recurrence`, but `None`
/// here means "leave alone", matching [`crate::UpdateDraft`]).
#[derive(Clone, Debug, Default, Serialize)]
pub struct UpdateCalendarEvent {
    /// Replace the title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Replace the description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Replace the location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Replace the stored metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Replace the confirmation state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EventStatus>,
    /// Switch to or from an all-day event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_day: Option<bool>,
    /// Replace the display start.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// Replace the display end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Replace the IANA timezone.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    /// Replace the duration interpretation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_mode: Option<DurationMode>,
    /// Replace the recurrence rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recurrence: Option<Recurrence>,
    /// Replace the attendee list.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attendees: Vec<CalendarAttendee>,
    /// Email invites to (new) attendees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_invites: Option<bool>,
}

/// Body for `respond_to_calendar_event`: the inbox's reply to an invite.
#[derive(Clone, Debug, Default, Serialize)]
pub struct RespondToCalendarEvent {
    /// The reply: `accepted`, `declined`, or `tentative`.
    pub status: String,
    /// A note sent along with the reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// Email the reply to the organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_reply: Option<bool>,
}

/// One page of occurrences from `get_agenda` or `list_event_instances`.
#[derive(Clone, Debug, Deserialize)]
pub struct CalendarEventList {
    /// Total occurrences in the window (not just this page).
    pub count: u64,
    /// This page of occurrences.
    #[serde(default)]
    pub events: Vec<CalendarEvent>,
    /// Cursor for the next page; `None` on the last page.
    #[serde(default)]
    pub next_page_token: Option<String>,
}

/// The response to an event create/update: the mutated event plus the
/// operation id (used by the API's event stream to ack the change).
#[derive(Clone, Debug, Deserialize)]
pub struct CalendarEventMutation {
    /// The event after the mutation.
    pub event: CalendarEvent,
    /// Id of the applied operation.
    #[serde(default)]
    pub operation_id: Option<String>,
}

/// The response to `delete_calendar_event`; `event` carries the deleted state
/// when the API returns it.
#[derive(Clone, Debug, Deserialize)]
pub struct DeleteCalendarEventResult {
    /// Id of the deletion operation, when the API issues one.
    #[serde(default)]
    pub deletion_id: Option<String>,
    /// The event as it was, when the API returns it.
    #[serde(default)]
    pub event: Option<CalendarEvent>,
}
