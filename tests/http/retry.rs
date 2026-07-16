use crate::common::*;
use std::time::Duration;

fn fast_client(server: &MockServer, max_retries: u32) -> agentmail::Client {
    agentmail::Client::new("test-key", server.uri()).with_retry_policy(agentmail::RetryPolicy {
        max_retries,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(5),
    })
}

#[tokio::test]
async fn retries_429_then_succeeds() {
    let server = MockServer::start().await;
    // First call 429, then the fallback mock serves 200.
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "inbox_id": "ib_1", "email": "x@agentmail.to"
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let client = fast_client(&server, 2);
    let inbox = client.org().get_inbox("ib_1").await.unwrap();
    assert_eq!(inbox.inbox_id, "ib_1");
}

#[tokio::test]
async fn honors_retry_after_zero() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(503).insert_header("retry-after", "0"))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "inbox_id": "ib_1", "email": "x@agentmail.to"
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let client = fast_client(&server, 2);
    assert_eq!(
        client.org().get_inbox("ib_1").await.unwrap().inbox_id,
        "ib_1"
    );
}

#[tokio::test]
async fn does_not_retry_4xx_client_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(400).set_body_string("bad"))
        .expect(1) // exactly one call: no retry on 400
        .mount(&server)
        .await;

    let client = fast_client(&server, 3);
    match client.org().get_inbox("ib_1").await {
        Err(agentmail::Error::Api { status, .. }) => assert_eq!(status.as_u16(), 400),
        other => panic!("expected Api 400, got {other:?}"),
    }
}

#[tokio::test]
async fn max_retries_zero_disables() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(429))
        .expect(1) // one call only, retries disabled
        .mount(&server)
        .await;

    let client = fast_client(&server, 0);
    assert!(matches!(
        client.org().get_inbox("ib_1").await,
        Err(agentmail::Error::Api { .. })
    ));
}

#[tokio::test]
async fn exhaustion_returns_last_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1"))
        .respond_with(ResponseTemplate::new(500))
        .expect(3) // initial + 2 retries
        .mount(&server)
        .await;

    let client = fast_client(&server, 2);
    match client.org().get_inbox("ib_1").await {
        Err(agentmail::Error::Api { status, .. }) => assert_eq!(status.as_u16(), 500),
        other => panic!("expected Api 500, got {other:?}"),
    }
}
