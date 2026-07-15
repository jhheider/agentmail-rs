//! HTTP-layer tests against a local mock server: auth header, decode paths,
//! error mapping, empty DELETE bodies, path encoding, pagination params.

use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn client() -> (MockServer, agentmail::Client) {
    let server = MockServer::start().await;
    let client = agentmail::Client::new("test-key", server.uri());
    (server, client)
}

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

    let list = client.list_inboxes().await.unwrap();
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

    match client.get_inbox("ib_missing").await {
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

    client.delete_inbox("ib_1").await.unwrap();
}

#[tokio::test]
async fn undecodable_2xx_body_is_decode_error() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>not json</html>"))
        .mount(&server)
        .await;

    match client.list_inboxes().await {
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
    client.get_inbox("a/b c").await.unwrap();
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

    let page = agentmail::Page {
        limit: Some(5),
        page_token: Some("tok_2".into()),
    };
    let list = client.list_messages_page("ib_1", page).await.unwrap();
    assert!(list.next_page_token.is_none());
}

#[tokio::test]
async fn send_message_posts_json_body() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/inboxes/ib_1/messages/send"))
        .and(wiremock::matchers::body_json(serde_json::json!({
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
