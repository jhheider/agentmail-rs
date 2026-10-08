//! Realtime websocket tests against a local in-process ws server (no TLS, no
//! network). Requires the `websockets` feature.

use agentmail::{Client, RealtimeEvent, Subscribe};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message as WsFrame;

#[tokio::test]
async fn subscribe_receive_other_and_close() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();

        // The client's first frame must be the subscribe request.
        let frame = match ws.next().await {
            Some(Ok(WsFrame::Text(text))) => {
                serde_json::from_str::<serde_json::Value>(&text).unwrap()
            }
            other => panic!("expected subscribe frame, got {other:?}"),
        };
        assert_eq!(frame["type"], "subscribe");
        assert_eq!(frame["inbox_ids"], serde_json::json!(["ib_1"]));
        assert_eq!(
            frame["event_types"],
            serde_json::json!(["message.received"])
        );

        ws.send(WsFrame::text(
            r#"{"type":"subscribed","inbox_ids":["ib_1"]}"#,
        ))
        .await
        .unwrap();
        ws.send(WsFrame::text(
            r#"{"type":"event","event_type":"message.received","event_id":"ev_1",
                "message":{"message_id":"m1","subject":"Hi","inbox_id":"ib_1","thread_id":"t1","to":["me@x.y"],"labels":["received"]},
                "thread":{"thread_id":"t1","labels":["received"],"message_count":1}}"#,
        ))
        .await
        .unwrap();
        // Unknown event types must surface as Other, not fail the stream.
        ws.send(WsFrame::text(
            r#"{"type":"event","event_type":"carrier.pigeon","data":1}"#,
        ))
        .await
        .unwrap();
        let _ = ws.close(None).await;
    });

    let client = Client::new("test-key", format!("http://{addr}"));
    let mut events = client
        .connect_realtime(&Subscribe {
            event_types: vec!["message.received".into()],
            inbox_ids: vec!["ib_1".into()],
            ..Default::default()
        })
        .await
        .unwrap();

    match events.next_event().await.unwrap() {
        RealtimeEvent::Subscribed(ack) => assert_eq!(ack.inbox_ids, vec!["ib_1"]),
        other => panic!("expected subscribed ack, got {other:?}"),
    }
    match events.next_event().await.unwrap() {
        RealtimeEvent::MessageReceived(event) => {
            assert_eq!(event.event_type.as_deref(), Some("message.received"));
            assert_eq!(event.message.subject.as_deref(), Some("Hi"));
            assert_eq!(event.thread.thread_id, "t1");
        }
        other => panic!("expected message.received, got {other:?}"),
    }
    match events.next_event().await.unwrap() {
        RealtimeEvent::Other(value) => assert_eq!(value["event_type"], "carrier.pigeon"),
        other => panic!("expected unknown event as Other, got {other:?}"),
    }
    assert!(matches!(
        events.next_event().await.unwrap(),
        RealtimeEvent::Closed
    ));
}

#[tokio::test]
async fn typed_payloads_decode_sent_and_calendar() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let _ = ws.next().await; // consume the subscribe frame
        ws.send(WsFrame::text(r#"{"type":"subscribed"}"#))
            .await
            .unwrap();
        ws.send(WsFrame::text(
            r#"{"type":"event","event_type":"message.sent","event_id":"ev_2",
                "send":{"inbox_id":"ib_1","thread_id":"t1","message_id":"m9",
                        "timestamp":"2026-01-01T00:00:00Z","recipients":["a@b.c"]}}"#,
        ))
        .await
        .unwrap();
        ws.send(WsFrame::text(
            r#"{"type":"event","event_type":"calendar.event.starting","event_id":"ev_3",
                "inbox_id":"ib_1","scheduled_at":"2026-03-02T14:55:00Z",
                "calendar_event":{"event_id":"cal_1","kind":"single","title":"Sync",
                    "status":"confirmed","all_day":false,"start":"2026-03-02T10:00:00",
                    "end":"2026-03-02T11:00:00","timezone":"UTC","duration_mode":"nominal",
                    "duration_value":60,"start_at":"2026-03-02T10:00:00Z",
                    "end_at":"2026-03-02T11:00:00Z","attendee_count":0,"uid":"u","sequence":0,
                    "source":"api","created_at":"2026-01-01T00:00:00Z",
                    "updated_at":"2026-01-01T00:00:00Z"}}"#,
        ))
        .await
        .unwrap();
        let _ = ws.close(None).await;
    });

    let client = Client::new("test-key", format!("http://{addr}"));
    let mut events = client
        .connect_realtime(&Subscribe::default())
        .await
        .unwrap();
    let _ = events.next_event().await.unwrap(); // subscribed ack

    match events.next_event().await.unwrap() {
        RealtimeEvent::MessageSent(event) => {
            assert_eq!(event.send.recipients, vec!["a@b.c"]);
            assert_eq!(event.send.message_id.as_deref(), Some("m9"));
        }
        other => panic!("expected message.sent, got {other:?}"),
    }
    match events.next_event().await.unwrap() {
        RealtimeEvent::CalendarEventStarting(event) => {
            assert_eq!(event.calendar_event.title, "Sync");
            assert_eq!(event.scheduled_at.as_deref(), Some("2026-03-02T14:55:00Z"));
        }
        other => panic!("expected calendar.event.starting, got {other:?}"),
    }
}

#[tokio::test]
async fn resubscribe_sends_new_filter() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = tokio_tungstenite::accept_async(stream).await.unwrap();
        let _ = ws.next().await; // first subscribe
        // A second subscribe frame arrives on resubscribe().
        let second = tokio::time::timeout(std::time::Duration::from_secs(5), ws.next())
            .await
            .expect("resubscribe frame within 5s");
        if let Some(Ok(WsFrame::Text(text))) = second {
            let v: serde_json::Value = serde_json::from_str(&text).unwrap();
            assert_eq!(v["type"], "subscribe");
            assert_eq!(v["pod_ids"], serde_json::json!(["pod_2"]));
        } else {
            panic!("expected second subscribe frame, got {second:?}");
        }
        let _ = ws.close(None).await;
    });

    let client = Client::new("test-key", format!("http://{addr}"));
    let mut events = client
        .connect_realtime(&Subscribe::default())
        .await
        .unwrap();
    events
        .subscribe(&Subscribe {
            pod_ids: vec!["pod_2".into()],
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(matches!(
        events.next_event().await.unwrap(),
        RealtimeEvent::Closed
    ));
}
