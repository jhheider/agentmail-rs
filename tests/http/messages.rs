use crate::common::*;

#[tokio::test]
async fn send_text_posts_minimal_body() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/send"))
        .and(body_json(serde_json::json!({
            "to": ["a@b.c"], "subject": "Hi", "text": "Body"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1", "thread_id": "t1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .inbox("ib_1")
        .send_text("a@b.c", "Hi", "Body")
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m1");
}

#[tokio::test]
async fn send_message_posts_json_body() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/send"))
        .and(body_json(serde_json::json!({
            "to": ["a@b.c"], "subject": "s", "text": "t",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1", "thread_id": "t1",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .inbox("ib_1")
        .send_message(agentmail::SendMessage {
            to: vec!["a@b.c".into()],
            subject: Some("s".into()),
            text: Some("t".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m1");
    assert_eq!(sent.thread_id, "t1");
}

#[tokio::test]
async fn reply_to_message_posts_json_body() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/m1/reply"))
        .and(body_json(serde_json::json!({ "text": "reply text" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m2", "thread_id": "t1",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .inbox("ib_1")
        .reply_to_message(
            "m1",
            agentmail::ReplyToMessage {
                text: Some("reply text".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m2");
}

#[tokio::test]
async fn reply_all_to_message_hits_correct_path() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/m1/reply-all"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m3", "thread_id": "t1",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .inbox("ib_1")
        .reply_all_to_message(
            "m1",
            agentmail::ReplyToMessage {
                text: Some("reply all".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(sent.message_id, "m3");
}

#[tokio::test]
async fn update_message_sends_patch_with_labels() {
    let (server, client) = client().await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1/messages/m1"))
        .and(body_json(serde_json::json!({
            "add_labels": ["read"],
            "remove_labels": ["unread"],
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m1",
            "labels": ["read"],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let msg = client
        .inbox("ib_1")
        .update_message(
            "m1",
            agentmail::UpdateMessage {
                add_labels: vec!["read".into()],
                remove_labels: vec!["unread".into()],
            },
        )
        .await
        .unwrap();
    assert!(msg.labels.contains(&"read".to_string()));
}

#[tokio::test]
async fn delete_message_returns_ok() {
    let (server, client) = client().await;
    Mock::given(method("DELETE"))
        .and(path("/v0/inboxes/ib_1/messages/m1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.inbox("ib_1").delete_message("m1").await.unwrap();
}

#[tokio::test]
async fn message_list_filters_sends_query_params() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .and(query_param("labels", "unread"))
        .and(query_param("before", "2026-07-15T00:00:00Z"))
        .and(query_param("after", "2026-07-14T00:00:00Z"))
        .and(query_param("limit", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 0, "messages": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let filters = agentmail::MessageListFilters {
        limit: Some(20),
        labels: vec!["unread".into()],
        before: Some("2026-07-15T00:00:00Z".into()),
        after: Some("2026-07-14T00:00:00Z".into()),
        ..Default::default()
    };
    let list = client.inbox("ib_1").list_messages(filters).await.unwrap();
    assert_eq!(list.count, 0);
}

#[tokio::test]
async fn message_list_filters_sends_from_to_subject() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .and(query_param("from", "a@b.c"))
        .and(query_param("to", "x@agentmail.to"))
        .and(query_param("subject", "invoice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "messages": [{"message_id": "m1"}],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let filters = agentmail::MessageListFilters {
        from: vec!["a@b.c".into()],
        to: vec!["x@agentmail.to".into()],
        subject: vec!["invoice".into()],
        ..Default::default()
    };
    let list = client.inbox("ib_1").list_messages(filters).await.unwrap();
    assert_eq!(list.messages[0].message_id, "m1");
}

#[tokio::test]
async fn search_messages_includes_query_param() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages/search"))
        .and(query_param("q", "urgent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "messages": [{"message_id": "m2", "subject": "URGENT: Meeting"}],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client
        .inbox("ib_1")
        .search_messages("urgent", Default::default())
        .await
        .unwrap();
    assert_eq!(list.count, 1);
}

#[tokio::test]
async fn search_messages_with_filters_and_pagination() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages/search"))
        .and(query_param("q", "hello"))
        .and(query_param("limit", "5"))
        .and(query_param("page_token", "tok"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 0, "messages": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let filters = agentmail::MessageListFilters {
        limit: Some(5),
        page_token: Some("tok".into()),
        ..Default::default()
    };
    let list = client
        .inbox("ib_1")
        .search_messages("hello", filters)
        .await
        .unwrap();
    assert_eq!(list.count, 0);
}

#[tokio::test]
async fn list_all_messages_drains_pages() {
    let (server, client) = client().await;
    // First page carries a token; second page ends it.
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .and(query_param("page_token", "p2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 2, "messages": [{"message_id": "m2"}]
        })))
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 2, "messages": [{"message_id": "m1"}], "next_page_token": "p2"
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let all = client
        .inbox("ib_1")
        .list_all_messages(Default::default())
        .await
        .unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].message_id, "m1");
    assert_eq!(all[1].message_id, "m2");
}
