use crate::common::*;

#[tokio::test]
async fn get_webhook_decodes_full_shape() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/webhooks/wh_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "webhook_id": "wh_1",
            "url": "https://example.com/hook",
            "event_types": ["message.received"],
            "inbox_ids": ["ib_1"],
            "pod_ids": [],
            "secret": "whsec_abc",
            "enabled": true,
            "client_id": "ref-1",
            "updated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let wh = client.org().get_webhook("wh_1").await.unwrap();
    assert_eq!(wh.webhook_id, "wh_1");
    assert!(wh.enabled);
    assert_eq!(wh.client_id.as_deref(), Some("ref-1"));
}

#[tokio::test]
async fn update_webhook_sends_delta_body() {
    let (server, client) = client().await;
    Mock::given(method("PATCH"))
        .and(path("/v0/webhooks/wh_1"))
        .and(body_json(serde_json::json!({
            "event_types": ["message.received", "message.bounced"],
            "add_inbox_ids": ["ib_2"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "webhook_id": "wh_1",
            "url": "https://example.com/hook",
            "secret": "whsec_abc",
            "enabled": true,
            "updated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let wh = client
        .org()
        .update_webhook(
            "wh_1",
            agentmail::UpdateWebhook {
                event_types: vec!["message.received".into(), "message.bounced".into()],
                add_inbox_ids: vec!["ib_2".into()],
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(wh.webhook_id, "wh_1");
}

#[tokio::test]
async fn get_and_update_webhook_headers() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/webhooks/wh_1/headers"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "header_names": ["X-Trace-Id", "X-Tenant"]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/webhooks/wh_1/headers"))
        .and(body_json(serde_json::json!({
            "headers": {"X-Trace-Id": "abc"},
            "remove_headers": ["X-Tenant"]
        })))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let names = client.org().get_webhook_headers("wh_1").await.unwrap();
    assert_eq!(names.header_names, vec!["X-Trace-Id", "X-Tenant"]);

    client
        .org()
        .update_webhook_headers(
            "wh_1",
            agentmail::UpdateWebhookHeaders {
                headers: Some(serde_json::json!({ "X-Trace-Id": "abc" })),
                remove_headers: vec!["X-Tenant".into()],
            },
        )
        .await
        .unwrap();
}
