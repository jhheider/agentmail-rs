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
        .send_text("ib_1", "a@b.c", "Hi", "Body")
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
        .send_message(
            "ib_1",
            agentmail::SendMessage {
                to: vec!["a@b.c".into()],
                subject: Some("s".into()),
                text: Some("t".into()),
                ..Default::default()
            },
        )
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
        .and(body_json(serde_json::json!({
            "text": "reply text",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message_id": "m2", "thread_id": "t1",
        })))
        .expect(1)
        .mount(&server)
        .await;

    let sent = client
        .reply_to_message(
            "ib_1",
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
        .reply_all_to_message(
            "ib_1",
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
        .update_message(
            "ib_1",
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

    client.delete_message("ib_1", "m1").await.unwrap();
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
    let list = client
        .list_messages_filtered("ib_1", filters)
        .await
        .unwrap();
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
    let list = client
        .list_messages_filtered("ib_1", filters)
        .await
        .unwrap();
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

    let list = client.search_messages("ib_1", "urgent").await.unwrap();
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
        .search_messages_page("ib_1", "hello", filters)
        .await
        .unwrap();
    assert_eq!(list.count, 0);
}

#[tokio::test]
async fn message_list_filters_default_is_empty_query() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 0, "messages": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client
        .list_messages_filtered("ib_1", agentmail::MessageListFilters::default())
        .await
        .unwrap();
    assert_eq!(list.count, 0);
}
