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
