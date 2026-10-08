//! Realtime inbox events over WebSocket (`websockets` feature).
//!
//! The API pushes the same events webhooks deliver (message lifecycle,
//! domain verification, calendar activity) over a WebSocket, so agents
//! without a public URL can receive mail without exposing an endpoint.
//!
//! Protocol (mirrors the official SDKs): connect to `{websocket}/v0` with
//! the API key as the `api_key` query parameter and a bearer `Authorization`
//! header, send one `{"type":"subscribe", ...}` frame, and read back a
//! `subscribed` acknowledgement followed by typed event frames until the
//! server closes the connection. [`RealtimeStream::next_event`] yields
//! [`RealtimeEvent`]s; unknown event types surface as
//! [`RealtimeEvent::Other`] rather than failing, so additions don't break
//! readers. There is no automatic reconnection: on
//! [`RealtimeEvent::Closed`], call [`Client::connect_realtime`] again.
//!
//! The websocket host is not derivable from the API host by scheme alone
//! (`https://api.agentmail.to` -> `wss://ws.agentmail.to`, EU:
//! `wss://ws.agentmail.eu`), so [`Client::connect_realtime`] derives it by
//! swapping the `api.` host label for `ws.` and overriding is available via
//! `AGENTMAIL_WEBSOCKET_URL` (with [`Client::from_env`]) or
//! [`Client::with_websocket_url`].

use crate::types::{CalendarAttendee, CalendarEvent, Domain, Message, Recurrence, Thread};
use crate::{Client, Error};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message as WsFrame;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

/// The production realtime host. See the module docs for how the URL is
/// derived and overridden.
pub const DEFAULT_WEBSOCKET_URL: &str = "wss://ws.agentmail.to";

/// A subscribe request: which events, and for which inboxes or pods. Empty
/// filters mean "everything the key can see".
#[derive(Clone, Debug, Default, Serialize)]
pub struct Subscribe {
    /// Event types to receive (e.g. `message.received`); empty means all.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub event_types: Vec<String>,
    /// Limit events to these inboxes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inbox_ids: Vec<String>,
    /// Limit events to these pods.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pod_ids: Vec<String>,
}

#[derive(Serialize)]
struct SubscribeFrame<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(flatten)]
    sub: &'a Subscribe,
}

/// The server's acknowledgement of a [`Subscribe`] frame.
#[derive(Clone, Debug, Deserialize)]
pub struct Subscribed {
    /// The active event filter (empty means all).
    #[serde(default)]
    pub event_types: Vec<String>,
    /// The active inbox filter.
    #[serde(default)]
    pub inbox_ids: Vec<String>,
    /// The active pod filter.
    #[serde(default)]
    pub pod_ids: Vec<String>,
}

/// A per-recipient delivery status, from bounce events.
#[derive(Clone, Debug, Deserialize)]
pub struct EventRecipient {
    /// The recipient address.
    #[serde(default)]
    pub address: Option<String>,
    /// Per-recipient status reported by the upstream provider.
    #[serde(default)]
    pub status: Option<String>,
}

/// A calendar event's state before an update, all fields optional.
#[derive(Clone, Debug, Deserialize)]
pub struct CalendarEventPrevious {
    /// Title before the update.
    #[serde(default)]
    pub title: Option<String>,
    /// Description before the update.
    #[serde(default)]
    pub description: Option<String>,
    /// Location before the update.
    #[serde(default)]
    pub location: Option<String>,
    /// Stored metadata before the update.
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Confirmation state before the update.
    #[serde(default)]
    pub status: Option<String>,
    /// Whether the event was all-day.
    #[serde(default)]
    pub all_day: Option<bool>,
    /// Display start before the update.
    #[serde(default)]
    pub start: Option<String>,
    /// Display end before the update.
    #[serde(default)]
    pub end: Option<String>,
    /// IANA timezone before the update.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Duration interpretation before the update.
    #[serde(default)]
    pub duration_mode: Option<String>,
    /// Recurrence rule before the update.
    #[serde(default)]
    pub recurrence: Option<Recurrence>,
    /// Attendee list before the update.
    #[serde(default)]
    pub attendees: Vec<CalendarAttendee>,
}

/// One event off the wire, decoded.
#[derive(Clone, Debug, Deserialize)]
pub struct MessageReceivedEvent {
    /// `message.received`, or `.spam` / `.blocked` / `.unauthenticated` for
    /// mail that was filtered.
    #[serde(default)]
    pub event_type: Option<String>,
    /// Server-issued event id.
    #[serde(default)]
    pub event_id: Option<String>,
    /// The received message.
    pub message: Message,
    /// The thread the message opened.
    pub thread: Thread,
}

macro_rules! envelope_event {
    ($name:ident { $payload:ident : $payload_ty:ty $(, $extra:ident : $extra_ty:ty)* }) => {
/// A decoded event frame; see the module docs for the protocol.
#[derive(Clone, Debug, Deserialize)]
pub struct $name {
            /// The event type on the wire (e.g. `message.sent`).
            #[serde(default)]
            pub event_type: Option<String>,
            /// Server-issued event id.
            #[serde(default)]
            pub event_id: Option<String>,
            /// Inbox the event concerns, when scoped to one.
            #[serde(default)]
            pub inbox_id: Option<String>,
            /// The event's payload.
            pub $payload: $payload_ty,
            $(/// Extra field carried by this event kind.
            #[serde(default)]
            pub $extra: $extra_ty,)*
        }
    };
}

/// Send/delivery details, from `message.sent` and `message.delivered`.
#[derive(Clone, Debug, Deserialize)]
pub struct DispatchPayload {
    /// Inbox that sent (or owns) the message.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Thread the message is filed under.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// The message.
    #[serde(default)]
    pub message_id: Option<String>,
    /// When the send/delivery happened (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Addresses the message reached.
    #[serde(default)]
    pub recipients: Vec<String>,
}

envelope_event!(MessageSentEvent {
    send: DispatchPayload
});
envelope_event!(MessageDeliveredEvent {
    delivery: DispatchPayload
});
envelope_event!(MessageOpenedEvent { open: OpenPayload });
envelope_event!(MessageBouncedEvent {
    bounce: BouncePayload
});
envelope_event!(MessageComplainedEvent {
    complaint: ComplaintPayload
});
envelope_event!(MessageRejectedEvent {
    reject: RejectPayload
});
envelope_event!(DomainVerifiedEvent { domain: Domain });
envelope_event!(CalendarEventCreatedEvent {
    calendar_event: CalendarEvent
});
envelope_event!(CalendarEventDeletedEvent {
    calendar_event: CalendarEvent
});
envelope_event!(CalendarEventRespondedEvent {
    calendar_event: CalendarEvent
});
envelope_event!(CalendarEventUpdatedEvent { calendar_event: CalendarEvent, previous: Option<CalendarEventPrevious> });
envelope_event!(CalendarEventStartingEvent { calendar_event: CalendarEvent, scheduled_at: Option<String> });
envelope_event!(CalendarEventEndingEvent { calendar_event: CalendarEvent, scheduled_at: Option<String> });

/// Recipient-open details, from `message.opened`.
#[derive(Clone, Debug, Deserialize)]
pub struct OpenPayload {
    /// Inbox the message was sent from.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Thread the message is filed under.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// The opened message.
    #[serde(default)]
    pub message_id: Option<String>,
    /// When the open was recorded (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Bounce details, from `message.bounced`.
#[derive(Clone, Debug, Deserialize)]
pub struct BouncePayload {
    /// Inbox that sent the message.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Thread the message is filed under.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// The bounced message.
    #[serde(default)]
    pub message_id: Option<String>,
    /// When the bounce was recorded (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Bounce class reported by the provider.
    #[serde(default)]
    pub sub_type: Option<String>,
    /// Per-recipient bounce statuses.
    #[serde(default)]
    pub recipients: Vec<EventRecipient>,
}

/// Complaint details, from `message.complained`.
#[derive(Clone, Debug, Deserialize)]
pub struct ComplaintPayload {
    /// Inbox that sent the message.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Thread the message is filed under.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// The complained-about message.
    #[serde(default)]
    pub message_id: Option<String>,
    /// When the complaint was recorded (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Complaint class reported by the provider.
    #[serde(default)]
    pub sub_type: Option<String>,
    /// Addresses that complained.
    #[serde(default)]
    pub recipients: Vec<String>,
}

/// Rejection details, from `message.rejected`.
#[derive(Clone, Debug, Deserialize)]
pub struct RejectPayload {
    /// Inbox that tried to send the message.
    #[serde(default)]
    pub inbox_id: Option<String>,
    /// Thread the message would have been filed under.
    #[serde(default)]
    pub thread_id: Option<String>,
    /// The rejected message.
    #[serde(default)]
    pub message_id: Option<String>,
    /// When the rejection happened (RFC 3339).
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Why the message was rejected.
    #[serde(default)]
    pub reason: Option<String>,
}

/// One frame from the stream.
#[derive(Clone, Debug)]
pub enum RealtimeEvent {
    /// The server accepted our [`Subscribe`] filter.
    Subscribed(Subscribed),
    /// Mail arrived (also fires for `.spam` / `.blocked` /
    /// `.unauthenticated` deliveries; see
    /// [`MessageReceivedEvent::event_type`]).
    MessageReceived(MessageReceivedEvent),
    /// The API accepted an outgoing message for delivery.
    MessageSent(MessageSentEvent),
    /// An upstream provider accepted the message.
    MessageDelivered(MessageDeliveredEvent),
    /// A recipient's client rendered the message's tracking pixel.
    MessageOpened(MessageOpenedEvent),
    /// A recipient's provider bounced the message.
    MessageBounced(MessageBouncedEvent),
    /// A recipient filed a spam complaint.
    MessageComplained(MessageComplainedEvent),
    /// The API refused to deliver an outgoing message.
    MessageRejected(MessageRejectedEvent),
    /// A domain finished verifying.
    DomainVerified(DomainVerifiedEvent),
    /// A calendar event was created.
    CalendarEventCreated(CalendarEventCreatedEvent),
    /// A calendar event was updated (`previous` holds the old state).
    CalendarEventUpdated(CalendarEventUpdatedEvent),
    /// A calendar event was deleted.
    CalendarEventDeleted(CalendarEventDeletedEvent),
    /// A calendar event is about to start.
    CalendarEventStarting(CalendarEventStartingEvent),
    /// A calendar event is about to end.
    CalendarEventEnding(CalendarEventEndingEvent),
    /// An attendee replied to an invite.
    CalendarEventResponded(CalendarEventRespondedEvent),
    /// A frame this client version doesn't know; the raw JSON is preserved.
    Other(serde_json::Value),
    /// A binary frame (passed through undecoded).
    Binary(Vec<u8>),
    /// The server closed the connection; reconnect with
    /// [`Client::connect_realtime`].
    Closed,
}

/// An open realtime stream. Send one [`Subscribe`] (done for you by
/// [`Client::connect_realtime`]), then poll [`RealtimeStream::next_event`].
pub struct RealtimeStream {
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
}

fn realtime_error(e: tokio_tungstenite::tungstenite::Error) -> Error {
    Error::Realtime(e.to_string())
}

impl RealtimeStream {
    /// Send a new subscribe filter on the open connection.
    pub async fn subscribe(&mut self, subscribe: &Subscribe) -> Result<(), Error> {
        let frame = SubscribeFrame {
            kind: "subscribe",
            sub: subscribe,
        };
        let text = serde_json::to_string(&frame).map_err(|e| Error::Decode {
            reason: e.to_string(),
            body: String::new(),
        })?;
        self.ws
            .send(WsFrame::Text(text.into()))
            .await
            .map_err(realtime_error)
    }

    /// Await the next frame. Control frames (ping/pong) are answered
    /// transparently; a server close yields [`RealtimeEvent::Closed`]
    /// (and then keeps yielding it).
    pub async fn next_event(&mut self) -> Result<RealtimeEvent, Error> {
        loop {
            match self.ws.next().await {
                Some(Ok(WsFrame::Text(text))) => {
                    let value: serde_json::Value =
                        serde_json::from_str(&text).map_err(|e| Error::Decode {
                            reason: e.to_string(),
                            body: text.to_string(),
                        })?;
                    return Ok(decode_event(value));
                }
                Some(Ok(WsFrame::Binary(bytes))) => {
                    return Ok(RealtimeEvent::Binary(bytes.to_vec()));
                }
                // Pings are answered by tungstenite; ignore the rest.
                Some(Ok(WsFrame::Ping(_) | WsFrame::Pong(_))) => continue,
                Some(Ok(WsFrame::Close(_))) | None => return Ok(RealtimeEvent::Closed),
                Some(Ok(WsFrame::Frame(_))) => continue,
                Some(Err(e)) => return Err(realtime_error(e)),
            }
        }
    }
}

fn decode_event(value: serde_json::Value) -> RealtimeEvent {
    match value.get("type").and_then(|t| t.as_str()) {
        Some("subscribed") => serde_json::from_value(value.clone())
            .map(|s: Subscribed| RealtimeEvent::Subscribed(s))
            .unwrap_or_else(|_| RealtimeEvent::Other(value)),
        Some("event") => decode_domain_event(value),
        _ => RealtimeEvent::Other(value),
    }
}

fn decode_domain_event(value: serde_json::Value) -> RealtimeEvent {
    use RealtimeEvent as E;
    let event_type = value
        .get("event_type")
        .and_then(|t| t.as_str())
        .unwrap_or("");
    let decoded = match event_type {
        "message.received"
        | "message.received.spam"
        | "message.received.blocked"
        | "message.received.unauthenticated" => {
            serde_json::from_value(value.clone()).map(E::MessageReceived)
        }
        "message.sent" => serde_json::from_value(value.clone()).map(E::MessageSent),
        "message.delivered" => serde_json::from_value(value.clone()).map(E::MessageDelivered),
        "message.opened" => serde_json::from_value(value.clone()).map(E::MessageOpened),
        "message.bounced" => serde_json::from_value(value.clone()).map(E::MessageBounced),
        "message.complained" => serde_json::from_value(value.clone()).map(E::MessageComplained),
        "message.rejected" => serde_json::from_value(value.clone()).map(E::MessageRejected),
        "domain.verified" => serde_json::from_value(value.clone()).map(E::DomainVerified),
        "calendar.event.created" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventCreated)
        }
        "calendar.event.updated" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventUpdated)
        }
        "calendar.event.deleted" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventDeleted)
        }
        "calendar.event.starting" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventStarting)
        }
        "calendar.event.ending" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventEnding)
        }
        "calendar.event.responded" => {
            serde_json::from_value(value.clone()).map(E::CalendarEventResponded)
        }
        _ => return RealtimeEvent::Other(value),
    };
    decoded.unwrap_or(RealtimeEvent::Other(value))
}

fn derive_websocket_url(base_url: &str) -> String {
    let (scheme, rest) = if let Some(rest) = base_url.strip_prefix("https://") {
        ("wss://", rest)
    } else if let Some(rest) = base_url.strip_prefix("http://") {
        ("ws://", rest)
    } else {
        return DEFAULT_WEBSOCKET_URL.to_string();
    };
    let host = rest.split('/').next().unwrap_or(rest);
    // api.agentmail.to -> ws.agentmail.to, x402.api.agentmail.to ->
    // x402.ws.agentmail.to (the `api` label swaps wherever it sits);
    // hosts without one (mock servers) pass through.
    let ws_host = host
        .split('.')
        .map(|label| if label == "api" { "ws" } else { label })
        .collect::<Vec<_>>()
        .join(".");
    format!("{scheme}{ws_host}")
}

impl Client {
    /// The websocket URL this client will dial, after derivation and any
    /// [`Client::with_websocket_url`] override.
    #[cfg(feature = "websockets")]
    pub fn websocket_url(&self) -> String {
        self.ws_url_override
            .clone()
            .unwrap_or_else(|| derive_websocket_url(&self.base_url))
    }

    /// Override the derived websocket host (e.g. for a proxy or a mock
    /// server on a different port).
    ///
    /// Requires the `websockets` feature (off by default).
    #[cfg(feature = "websockets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websockets")))]
    pub fn with_websocket_url(mut self, url: impl Into<String>) -> Self {
        self.ws_url_override = Some(url.into().trim_end_matches('/').to_string());
        self
    }

    /// Open the realtime event stream and send `subscribe`.
    ///
    /// The URL is [`Client::websocket_url`] + `/v0`, with the API key both in
    /// the `api_key` query parameter and the `Authorization` header (the
    /// official SDKs do the same). No automatic reconnection: on
    /// [`RealtimeEvent::Closed`], call this again.
    ///
    /// Requires the `websockets` feature (off by default).
    #[cfg(feature = "websockets")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websockets")))]
    pub async fn connect_realtime(&self, subscribe: &Subscribe) -> Result<RealtimeStream, Error> {
        let url = format!(
            "{}/v0?api_key={}",
            self.websocket_url(),
            crate::util::urlish(&self.api_key)
        );
        let mut request = url.as_str().into_client_request().map_err(realtime_error)?;
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", self.api_key)
                .parse()
                .map_err(|e| Error::Realtime(format!("invalid authorization header: {e}")))?,
        );
        let (ws, _response) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(realtime_error)?;
        let mut stream = RealtimeStream { ws };
        stream.subscribe(subscribe).await?;
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_derivation_matches_upstream_environments() {
        // From the official SDK's environment table.
        assert_eq!(
            derive_websocket_url("https://api.agentmail.to"),
            "wss://ws.agentmail.to"
        );
        assert_eq!(
            derive_websocket_url("https://api.agentmail.eu"),
            "wss://ws.agentmail.eu"
        );
        assert_eq!(
            derive_websocket_url("https://x402.api.agentmail.to"),
            "wss://x402.ws.agentmail.to"
        );
        // Mock servers keep their host.
        assert_eq!(
            derive_websocket_url("http://127.0.0.1:8080"),
            "ws://127.0.0.1:8080"
        );
    }

    #[test]
    fn subscribe_frame_has_const_type_and_skips_empty() {
        let frame = serde_json::to_value(SubscribeFrame {
            kind: "subscribe",
            sub: &Subscribe {
                inbox_ids: vec!["ib_1".into()],
                ..Default::default()
            },
        })
        .unwrap();
        assert_eq!(
            frame,
            serde_json::json!({"type": "subscribe", "inbox_ids": ["ib_1"]})
        );
    }

    #[test]
    fn unknown_event_types_decode_as_other() {
        let event = decode_event(serde_json::json!({
            "type": "event", "event_type": "carrier.pigeon", "data": 1
        }));
        assert!(matches!(event, RealtimeEvent::Other(_)));
    }
}
