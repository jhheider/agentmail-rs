use crate::common::*;

#[tokio::test]
async fn create_draft_posts_json_body() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/drafts"))
        .and(body_json(serde_json::json!({
            "to": ["a@b.c"], "subject": "Draft", "text": "body",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "draft_id": "d1",
            "inbox_id": "ib_1",
            "to": ["a@b.c"],
            "subject": "Draft",
            "text": "body",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let draft = client
        .inbox("ib_1")
        .create_draft(agentmail::CreateDraft {
            to: vec!["a@b.c".into()],
            subject: Some("Draft".into()),
            text: Some("body".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(draft.draft_id, "d1");
}

#[tokio::test]
async fn list_drafts_decodes_first_page() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/drafts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "drafts": [{"draft_id": "d1", "subject": "Hi"}],
        })))
        .mount(&server)
        .await;

    let list = client
        .inbox("ib_1")
        .list_drafts(Default::default())
        .await
        .unwrap();
    assert_eq!(list.count, 1);
    assert_eq!(list.drafts[0].draft_id, "d1");
}

#[tokio::test]
async fn get_draft_returns_full_draft() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/drafts/d1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "draft_id": "d1",
            "subject": "Draft",
            "text": "body",
            "to": ["a@b.c"],
        })))
        .mount(&server)
        .await;

    let draft = client.inbox("ib_1").get_draft("d1").await.unwrap();
    assert_eq!(draft.draft_id, "d1");
}

#[tokio::test]
async fn update_draft_sends_patch() {
    let (server, client) = client().await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1/drafts/d1"))
        .and(body_json(serde_json::json!({
            "subject": "Updated",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "draft_id": "d1",
            "subject": "Updated",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let draft = client
        .inbox("ib_1")
        .update_draft(
            "d1",
            agentmail::UpdateDraft {
                subject: Some("Updated".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(draft.subject.as_deref(), Some("Updated"));
}

#[tokio::test]
async fn delete_draft_returns_ok() {
    let (server, client) = client().await;
    Mock::given(method("DELETE"))
        .and(path("/v0/inboxes/ib_1/drafts/d1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.inbox("ib_1").delete_draft("d1").await.unwrap();
}

#[tokio::test]
async fn send_draft_posts_and_returns_sent_message() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/drafts/d1/send"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1", "thread_id": "t1",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client.inbox("ib_1").send_draft("d1").await.unwrap();
    assert_eq!(sent.message_id, "m1");
}

#[tokio::test]
async fn create_draft_skips_empty_in_reply_to() {
    let body = serde_json::to_value(agentmail::CreateDraft {
        to: vec!["a@b.c".into()],
        text: Some("body".into()),
        ..Default::default()
    })
    .unwrap();
    let obj = body.as_object().unwrap();
    assert!(!obj.contains_key("in_reply_to"));
}

#[tokio::test]
async fn update_draft_serializes_attachment_delta() {
    let body = serde_json::to_value(agentmail::UpdateDraft {
        add_attachments: vec![agentmail::SendAttachment {
            filename: Some("new.txt".into()),
            content: Some("aGk=".into()),
            ..Default::default()
        }],
        remove_attachments: vec!["att_1".into()],
        reply_to: Some(vec!["reply@b.c".into()]),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        body["add_attachments"][0]["filename"],
        serde_json::json!("new.txt")
    );
    assert_eq!(body["remove_attachments"], serde_json::json!(["att_1"]));
    assert_eq!(body["reply_to"], serde_json::json!(["reply@b.c"]));
    // Unset delta fields stay out of the body.
    assert!(body.get("add_labels").is_none());
}

#[tokio::test]
async fn draft_decodes_send_status_and_references() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/drafts/d1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "draft_id": "d1", "inbox_id": "ib_1",
            "labels": ["draft"], "preview": "Hi…",
            "reply_to": ["reply@b.c"], "references": ["<m1@b.c>"],
            "send_status": "scheduled", "send_at": "2026-03-01T09:00:00Z",
            "client_id": "cid_1", "updated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let draft = client.inbox("ib_1").get_draft("d1").await.unwrap();
    assert_eq!(draft.send_status.as_deref(), Some("scheduled"));
    assert_eq!(draft.references, vec!["<m1@b.c>"]);
    assert_eq!(draft.preview.as_deref(), Some("Hi…"));
}
