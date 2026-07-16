use crate::common::*;

#[tokio::test]
async fn create_list_delete_api_key() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/api-keys"))
        .and(body_json(serde_json::json!({ "name": "ci" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "api_key_id": "key_1", "api_key": "am_secret_xyz",
            "prefix": "am_secr", "name": "ci",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/api-keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "api_keys": [{"api_key_id": "key_1", "name": "ci"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v0/api-keys/key_1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let created = client
        .create_api_key(agentmail::CreateApiKey {
            name: Some("ci".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(created.api_key, "am_secret_xyz");
    assert_eq!(client.list_api_keys().await.unwrap().count, 1);
    client.delete_api_key("key_1").await.unwrap();
}

#[tokio::test]
async fn get_organization_counts_and_limits() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/organizations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_id": "org_1",
            "inbox_count": 2, "domain_count": 1,
            "inbox_limit": 3, "domain_limit": 5,
            "updated_at": "2026-01-01T00:00:00Z", "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;

    let org = client.get_organization().await.unwrap();
    assert_eq!(org.organization_id, "org_1");
    assert_eq!(org.inbox_count, Some(2));
    assert_eq!(org.inbox_limit, Some(3));
}

#[tokio::test]
async fn agent_sign_up_and_verify() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/agent/sign-up"))
        .and(body_json(serde_json::json!({
            "human_email": "me@example.com", "username": "my-agent"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "organization_id": "org_1", "inbox_id": "ib_1", "api_key": "am_new"
        })))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/agent/verify"))
        .and(body_json(serde_json::json!({ "otp_code": "123456" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "verified": true
        })))
        .mount(&server)
        .await;

    let signup = client
        .agent_sign_up(agentmail::AgentSignup {
            human_email: "me@example.com".into(),
            username: "my-agent".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(signup.api_key, "am_new");
    assert!(client.agent_verify("123456").await.unwrap().verified);
}
