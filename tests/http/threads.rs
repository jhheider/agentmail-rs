use crate::common::*;

#[tokio::test]
async fn list_threads_decodes_first_page() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/threads"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "threads": [{
                "thread_id": "th_1", "inbox_id": "ib_1", "labels": ["unread"],
                "senders": ["a@b.c"], "recipients": ["x@agentmail.to"],
                "subject": "hi", "preview": "…", "message_count": 2,
                "surprise_field": true
            }],
            "next_page_token": null
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client.list_threads("ib_1").await.unwrap();
    assert_eq!(list.count, 1);
    assert_eq!(list.threads[0].thread_id, "th_1");
    assert_eq!(list.threads[0].message_count, Some(2));
    assert!(list.threads[0].messages.is_empty());
}

#[tokio::test]
async fn list_threads_filtered_sends_query_params() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/threads"))
        .and(query_param("limit", "5"))
        .and(query_param("labels", "unread"))
        .and(query_param("senders", "a@b.c"))
        .and(query_param("include_spam", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 0, "threads": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let filters = agentmail::ThreadListFilters {
        limit: Some(5),
        labels: vec!["unread".into()],
        senders: vec!["a@b.c".into()],
        include_spam: Some(true),
        ..Default::default()
    };
    let list = client.list_threads_filtered("ib_1", filters).await.unwrap();
    assert_eq!(list.count, 0);
}

#[tokio::test]
async fn search_threads_includes_query_param() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/threads/search"))
        .and(query_param("q", "invoice"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "threads": [{
                "thread_id": "th_2", "inbox_id": "ib_1", "labels": [],
                "senders": [], "recipients": [],
                "highlights": {"subject": ["<em>invoice</em>"]}
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client.search_threads("ib_1", "invoice").await.unwrap();
    assert_eq!(list.threads[0].thread_id, "th_2");
    assert!(list.threads[0].highlights.is_some());
}

#[tokio::test]
async fn get_thread_returns_messages() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/threads/th_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "thread_id": "th_1", "inbox_id": "ib_1", "labels": [],
            "senders": [], "recipients": [],
            "messages": [{
                "message_id": "m1", "thread_id": "th_1", "from": "a@b.c",
                "subject": "hi", "timestamp": "2026-01-01T00:00:00Z"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let thread = client.get_thread("ib_1", "th_1").await.unwrap();
    assert_eq!(thread.messages.len(), 1);
    assert_eq!(thread.messages[0].message_id, "m1");
}

#[tokio::test]
async fn update_thread_sends_patch_with_labels() {
    let (server, client) = client().await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1/threads/th_1"))
        .and(body_json(serde_json::json!({ "add_labels": ["archived"] })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "thread_id": "th_1", "labels": ["archived"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let updated = client
        .update_thread(
            "ib_1",
            "th_1",
            agentmail::UpdateThread {
                add_labels: vec!["archived".into()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(updated.labels.contains(&"archived".to_string()));
}

#[tokio::test]
async fn delete_thread_returns_ok() {
    let (server, client) = client().await;
    Mock::given(method("DELETE"))
        .and(path("/v0/inboxes/ib_1/threads/th_1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.delete_thread("ib_1", "th_1").await.unwrap();
}
