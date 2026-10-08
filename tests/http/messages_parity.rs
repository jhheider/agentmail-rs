use crate::common::*;

#[tokio::test]
async fn forward_message_posts_to_forward_path() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/m1/forward"))
        .and(body_json(serde_json::json!({ "to": ["x@y.z"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m2", "thread_id": "t1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .inbox("ib_1")
        .forward_message(
            "m1",
            agentmail::SendMessage {
                to: vec!["x@y.z".into()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m2");
}

#[tokio::test]
async fn send_message_serializes_attachments_and_reply_to() {
    let body = serde_json::to_value(agentmail::SendMessage {
        to: vec!["a@b.c".into()],
        text: Some("hi".into()),
        reply_to: vec!["reply@b.c".into()],
        attachments: vec![agentmail::SendAttachment {
            filename: Some("a.txt".into()),
            content: Some("aGk=".into()),
            ..Default::default()
        }],
        ..Default::default()
    })
    .unwrap();
    assert_eq!(body["reply_to"], serde_json::json!(["reply@b.c"]));
    assert_eq!(body["attachments"][0]["filename"], "a.txt");
    // Empty optional fields stay out of the body.
    assert!(body.get("headers").is_none());
}

#[tokio::test]
async fn get_raw_message_and_download() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages/m1/raw"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1", "size": 42,
            "download_url": format!("{}/raw/m1", server.uri()),
            "expires_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/raw/m1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"From: a@b.c\r\n".to_vec()))
        .mount(&server)
        .await;

    let raw = client.inbox("ib_1").get_raw_message("m1").await.unwrap();
    assert_eq!(raw.size, Some(42));
    let bytes = client.download_raw(&raw).await.unwrap();
    assert!(bytes.starts_with(b"From:"));
}

#[tokio::test]
async fn batch_get_and_update() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/batch-get"))
        .and(body_json(serde_json::json!({ "message_ids": ["m1", "m2"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "limit": 100, "count": 2,
            "messages": [
                {"message_id": "m1", "thread_id": "t1", "from": "a@b.c", "timestamp": "2026-01-01T00:00:00Z"},
                {"message_id": "m2", "thread_id": "t1", "from": "a@b.c", "timestamp": "2026-01-01T00:00:00Z"}
            ]
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/batch-update"))
        .and(body_json(serde_json::json!({
            "message_ids": ["m1", "m2"], "add_labels": ["read"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "limit": 100, "count": 2,
            "updates": [
                {"message_id": "m1", "labels": ["read"]},
                {"message_id": "m2", "labels": ["read"]}
            ]
        })))
        .mount(&server)
        .await;

    let got = client
        .inbox("ib_1")
        .batch_get_messages(vec!["m1".into(), "m2".into()])
        .await
        .unwrap();
    assert_eq!(got.count, 2);
    assert_eq!(got.messages.len(), 2);

    let updated = client
        .inbox("ib_1")
        .batch_update_messages(agentmail::BatchUpdateMessages {
            message_ids: vec!["m1".into(), "m2".into()],
            add_labels: vec!["read".into()],
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(updated.updates.len(), 2);
}

#[tokio::test]
async fn message_decodes_current_spec_fields() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages/m1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1", "inbox_id": "ib_1", "thread_id": "t1",
            "labels": ["received"], "timestamp": "2026-01-01T00:00:00Z",
            "from": "a@b.c", "to": ["me@agentmail.to"],
            "cc": ["c1@b.c"],
            "reply_to": ["noreply@b.c"],
            "subject": "Hi", "preview": "Hel", "text": "Hello",
            "extracted_text": "PDF body",
            "in_reply_to": "<m0@b.c>", "references": ["<m0@b.c>"],
            "headers": {"X-Custom": "1"},
            "calendar_event_id": "ev_1",
            "size": 2048,
            "updated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let m = client.inbox("ib_1").get_message("m1").await.unwrap();
    assert_eq!(m.cc, vec!["c1@b.c"]);
    assert_eq!(m.reply_to, vec!["noreply@b.c"]);
    assert_eq!(m.extracted_text.as_deref(), Some("PDF body"));
    assert_eq!(m.in_reply_to.as_deref(), Some("<m0@b.c>"));
    assert_eq!(m.references, vec!["<m0@b.c>"]);
    assert_eq!(m.calendar_event_id.as_deref(), Some("ev_1"));
    assert_eq!(m.size, Some(2048));
    assert_eq!(m.created_at.as_deref(), Some("2026-01-01T00:00:00Z"));
}

#[tokio::test]
async fn send_serializes_track_opens_and_reply_flag() {
    let body = serde_json::to_value(agentmail::SendMessage {
        to: vec!["a@b.c".into()],
        text: Some("hi".into()),
        track_opens: Some(true),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(body["track_opens"], serde_json::json!(true));

    let reply = serde_json::to_value(agentmail::ReplyToMessage {
        to: vec!["a@b.c".into()],
        reply_all: Some(true),
        text: Some("hi".into()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(reply["reply_all"], serde_json::json!(true));
    assert_eq!(reply["to"], serde_json::json!(["a@b.c"]));
}
