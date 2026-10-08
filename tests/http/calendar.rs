use crate::common::*;

fn event_json(id: &str) -> serde_json::Value {
    event_titled(id, "Sync")
}

fn event_titled(id: &str, title: &str) -> serde_json::Value {
    serde_json::json!({
        "event_id": id, "kind": "single", "title": title,
        "status": "confirmed", "all_day": false,
        "start": "2026-03-02T10:00:00", "end": "2026-03-02T11:00:00",
        "timezone": "America/New_York",
        "duration_mode": "nominal", "duration_value": 60,
        "start_at": "2026-03-02T15:00:00Z", "end_at": "2026-03-02T16:00:00Z",
        "attendee_count": 1,
        "attendees": [{"email": "me@agentmail.to", "status": "needs_action"}],
        "uid": "uid-1", "sequence": 0, "source": "api",
        "created_at": "2026-02-01T00:00:00Z", "updated_at": "2026-02-01T00:00:00Z"
    })
}

#[tokio::test]
async fn get_and_update_calendar_settings() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/calendar"))
        .and(query_param("consistency", "primary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "inbox_id": "ib_1", "timezone": "UTC", "event_count": 3,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z", "etag": "e1"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1/calendar"))
        .and(header("If-Match", "e1"))
        .and(body_json(
            serde_json::json!({ "timezone": "Pacific/Auckland" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "inbox_id": "ib_1", "timezone": "Pacific/Auckland", "event_count": 3,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z", "etag": "e2"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let cal = client
        .inbox("ib_1")
        .get_calendar(Some(agentmail::Consistency::Primary))
        .await
        .unwrap();
    assert_eq!(cal.timezone, "UTC");
    assert_eq!(cal.etag, "e1");

    let updated = client
        .inbox("ib_1")
        .update_calendar("Pacific/Auckland", Some("e1"))
        .await
        .unwrap();
    assert_eq!(updated.timezone, "Pacific/Auckland");
}

#[tokio::test]
async fn create_get_update_delete_event() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/calendar/events"))
        .and(body_json(serde_json::json!({
            "title": "Sync", "start": "2026-03-02T10:00:00",
            "end": "2026-03-02T11:00:00",
            "recurrence": {"rule": "FREQ=WEEKLY"},
            "attendees": [{"email": "me@agentmail.to"}],
            "send_invites": true
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "event": event_json("ev_1"), "operation_id": "op_1"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1/calendar/events/ev_1"))
        .and(query_param("mode", "future"))
        .and(header("If-Match", "e9"))
        .and(body_json(serde_json::json!({ "title": "Sync (moved)" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event": event_titled("ev_1", "Sync (moved)"), "operation_id": "op_2"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v0/inboxes/ib_1/calendar/events/ev_1"))
        .and(query_param("mode", "single"))
        .and(query_param("send_invites", "true"))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "deletion_id": "del_1", "event": event_json("ev_1")
        })))
        .expect(1)
        .mount(&server)
        .await;

    let created = client
        .inbox("ib_1")
        .create_calendar_event(agentmail::CreateCalendarEvent {
            title: "Sync".into(),
            start: "2026-03-02T10:00:00".into(),
            end: "2026-03-02T11:00:00".into(),
            recurrence: Some(agentmail::Recurrence {
                rule: "FREQ=WEEKLY".into(),
                ..Default::default()
            }),
            attendees: vec![agentmail::CalendarAttendee {
                email: "me@agentmail.to".into(),
                ..Default::default()
            }],
            send_invites: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(created.event.title, "Sync");
    assert_eq!(created.operation_id.as_deref(), Some("op_1"));

    let updated = client
        .inbox("ib_1")
        .update_calendar_event(
            "ev_1",
            agentmail::UpdateCalendarEvent {
                title: Some("Sync (moved)".into()),
                ..Default::default()
            },
            Some(agentmail::MutationMode::Future),
            Some("e9"),
        )
        .await
        .unwrap();
    assert_eq!(updated.event.title, "Sync (moved)");

    let deleted = client
        .inbox("ib_1")
        .delete_calendar_event(
            "ev_1",
            Some(agentmail::MutationMode::Single),
            Some(true),
            None,
        )
        .await
        .unwrap();
    assert_eq!(deleted.deletion_id.as_deref(), Some("del_1"));
    assert!(deleted.event.is_some());
}

#[tokio::test]
async fn agenda_and_instances_paginate_with_window() {
    let (server, client) = client().await;
    for p in [
        "/v0/inboxes/ib_1/calendar/agenda",
        "/v0/inboxes/ib_1/calendar/events/ev_1/instances",
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            // First-mounted mocks win in wiremock, and query_param allows
            // extras; cap at one match so the page-token call reaches the
            // second mock.
            .and(query_param("after", "2026-03-01T00:00:00Z"))
            .and(query_param("before", "2026-03-08T00:00:00Z"))
            .and(query_param("include_overlapping", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2, "limit": 1, "next_page_token": "tok",
                "events": [event_json("ev_1")]
            })))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(p))
            .and(query_param("page_token", "tok"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2, "limit": 1, "events": [event_json("ev_2")]
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    let q = agentmail::CalendarEventsQuery {
        after: Some("2026-03-01T00:00:00Z".into()),
        before: Some("2026-03-08T00:00:00Z".into()),
        include_overlapping: Some(true),
        ..Default::default()
    };
    let agenda = client
        .inbox("ib_1")
        .list_all_agenda(q.clone())
        .await
        .unwrap();
    assert_eq!(agenda.len(), 2);
    assert_eq!(agenda[1].event_id, "ev_2");

    let instances = client
        .inbox("ib_1")
        .list_all_calendar_event_instances("ev_1", q)
        .await
        .unwrap();
    assert_eq!(instances.len(), 2);
}

#[tokio::test]
async fn respond_to_event_sends_if_match() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/calendar/events/ev_1/respond"))
        .and(header("If-Match", "e5"))
        .and(body_json(serde_json::json!({
            "status": "accepted", "comment": "See you there", "send_reply": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event": event_json("ev_1"), "operation_id": "op_3"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client
        .inbox("ib_1")
        .respond_to_calendar_event(
            "ev_1",
            agentmail::RespondToCalendarEvent {
                status: "accepted".into(),
                comment: Some("See you there".into()),
                send_reply: Some(true),
            },
            Some("e5"),
        )
        .await
        .unwrap();
    assert_eq!(result.event.attendees[0].email, "me@agentmail.to");
}
