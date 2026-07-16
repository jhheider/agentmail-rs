use crate::common::*;

#[tokio::test]
async fn update_inbox_sends_patch() {
    let (server, client) = client().await;
    Mock::given(method("PATCH"))
        .and(path("/v0/inboxes/ib_1"))
        .and(body_json(serde_json::json!({ "display_name": "Renamed" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pod_id": "pod_1", "inbox_id": "ib_1", "email": "x@agentmail.to",
            "display_name": "Renamed",
            "updated_at": "2026-01-01T00:00:00Z", "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let inbox = client
        .org()
        .update_inbox(
            "ib_1",
            agentmail::UpdateInbox {
                display_name: Some("Renamed".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_eq!(inbox.display_name.as_deref(), Some("Renamed"));
}

#[tokio::test]
async fn auth_me_returns_identity() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/auth/me"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "scope_type": "inbox",
            "scope_id": "ib_1",
            "organization_id": "org_1",
            "inbox_id": "ib_1",
            "api_key_id": "key_1"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let me = client.auth_me().await.unwrap();
    assert_eq!(me.scope_type, agentmail::ScopeType::Inbox);
    assert_eq!(me.organization_id, "org_1");
    assert_eq!(me.inbox_id.as_deref(), Some("ib_1"));
}
