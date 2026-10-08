use crate::common::*;

#[tokio::test]
async fn sends_bearer_auth_and_decodes_success() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "inboxes": [{"inbox_id": "ib_1", "email": "x@agentmail.to"}],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client.org().list_inboxes(Default::default()).await.unwrap();
    assert_eq!(list.count, 1);
    assert_eq!(list.inboxes[0].email, "x@agentmail.to");
}

#[tokio::test]
async fn non_2xx_maps_to_api_error() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string(r#"{"error":"not found"}"#))
        .mount(&server)
        .await;

    match client.org().get_inbox("ib_missing").await {
        Err(agentmail::Error::Api { status, body }) => {
            assert_eq!(status.as_u16(), 404);
            assert!(body.contains("not found"));
        }
        other => panic!("expected Error::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn empty_delete_body_is_ok() {
    let (server, client) = client().await;
    Mock::given(method("DELETE"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client.org().delete_inbox("ib_1").await.unwrap();
}

#[tokio::test]
async fn undecodable_2xx_body_is_decode_error() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>not json</html>"))
        .mount(&server)
        .await;

    match client.org().list_inboxes(Default::default()).await {
        Err(agentmail::Error::Decode { body, .. }) => assert!(body.contains("not json")),
        other => panic!("expected Error::Decode, got {other:?}"),
    }
}

#[tokio::test]
async fn path_segments_are_percent_encoded_on_the_wire() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/a%2Fb%20c"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "inbox_id": "ib_1", "email": "x@agentmail.to",
        })))
        .expect(1)
        .mount(&server)
        .await;

    // A slash or space in an id must not change the route shape.
    client.org().get_inbox("a/b c").await.unwrap();
}

#[tokio::test]
async fn pagination_params_reach_the_query_string() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages"))
        .and(query_param("limit", "5"))
        .and(query_param("page_token", "tok_2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 0, "messages": [],
        })))
        .expect(1)
        .mount(&server)
        .await;

    let filters = agentmail::MessageListFilters {
        limit: Some(5),
        page_token: Some("tok_2".into()),
        ..Default::default()
    };
    let list = client.inbox("ib_1").list_messages(filters).await.unwrap();
    assert!(list.next_page_token.is_none());
}

#[tokio::test]
async fn scope_prefixes_org_inbox_pod() {
    let (server, client) = client().await;
    // The same resource (threads) at all three scopes, proving base()/the
    // generic Scoped impl route to the right prefix.
    for p in [
        "/v0/threads",
        "/v0/inboxes/ib_1/threads",
        "/v0/pods/pod_1/threads",
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 0, "threads": []
            })))
            .expect(1)
            .mount(&server)
            .await;
    }
    client.org().list_threads(Default::default()).await.unwrap();
    client
        .inbox("ib_1")
        .list_threads(Default::default())
        .await
        .unwrap();
    client
        .pod("pod_1")
        .list_threads(Default::default())
        .await
        .unwrap();
}

#[tokio::test]
async fn inbox_search_at_org_and_pod_scopes() {
    let (server, client) = client().await;
    for p in ["/v0/inboxes/search", "/v0/pods/pod_1/inboxes/search"] {
        Mock::given(method("GET"))
            .and(path(p))
            .and(query_param("q", "velvet"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "inboxes": [{
                    "inbox_id": "ib_9", "email": "velvet@agentmail.to",
                    "status": "paused", "metadata": {"team": "ops"},
                    "updated_at": "2026-01-01T00:00:00Z",
                    "created_at": "2026-01-01T00:00:00Z"
                }]
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    for found in [
        client
            .org()
            .search_inboxes("velvet", Default::default())
            .await,
        client
            .pod("pod_1")
            .search_inboxes("velvet", Default::default())
            .await,
    ] {
        let list = found.unwrap();
        assert_eq!(list.count, 1);
        assert_eq!(list.inboxes[0].email, "velvet@agentmail.to");
        assert_eq!(list.inboxes[0].status.as_deref(), Some("paused"));
        assert_eq!(list.inboxes[0].metadata.as_ref().unwrap()["team"], "ops");
    }
}

#[tokio::test]
async fn authorize_inbox_exchanges_auth_token() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/authorize"))
        .and(body_json(serde_json::json!({
            "auth_token": "tok_123", "accept_disclosure": true
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "api_key_id": "key_9",
            "instructions": "Authorization complete. Return to browser."
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client
        .inbox("ib_1")
        .authorize(agentmail::AuthorizeInbox {
            auth_token: "tok_123".into(),
            accept_disclosure: Some(true),
        })
        .await
        .unwrap();
    assert_eq!(result.api_key_id, "key_9");
}
