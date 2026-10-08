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
        .org()
        .create_api_key(agentmail::CreateApiKey {
            name: Some("ci".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(created.api_key, "am_secret_xyz");
    assert_eq!(
        client
            .org()
            .list_api_keys(Default::default())
            .await
            .unwrap()
            .count,
        1
    );
    client.org().delete_api_key("key_1").await.unwrap();
}

#[tokio::test]
async fn get_and_update_api_key() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/api-keys/key_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "bearer", "api_key_id": "key_1", "prefix": "am_live",
            "name": "ci", "permissions": {"message_send": true},
            "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-02T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/api-keys/key_1"))
        .and(body_json(serde_json::json!({
            "name": "renamed",
            "permissions": {"message_send": true, "draft_create": true}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "type": "bearer", "api_key_id": "key_1", "name": "renamed"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let key = client.org().get_api_key("key_1").await.unwrap();
    assert_eq!(key.key_type.as_deref(), Some("bearer"));
    assert!(key.permissions.as_ref().unwrap()["message_send"]);
    let updated = client
        .org()
        .update_api_key(
            "key_1",
            agentmail::UpdateApiKey {
                name: Some("renamed".into()),
                permissions: Some(
                    [
                        ("message_send".to_string(), true),
                        ("draft_create".to_string(), true),
                    ]
                    .into_iter()
                    .collect(),
                ),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.name.as_deref(), Some("renamed"));
}

#[tokio::test]
async fn agent_attach_human_posts_email() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/agent/human"))
        .and(body_json(
            serde_json::json!({ "human_email": "me@example.com" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "human_email": "me@example.com",
            "instructions": "Check your inbox."
        })))
        .expect(1)
        .mount(&server)
        .await;

    let result = client
        .agent_attach_human(agentmail::AgentAttachHuman {
            human_email: "me@example.com".into(),
        })
        .await
        .unwrap();
    assert_eq!(result.human_email, "me@example.com");
    assert!(result.instructions.contains("Check"));
}

#[tokio::test]
async fn list_get_update_accounts() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/accounts"))
        .and(query_param("ascending", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "limit": 50,
            "accounts": [{
                "account_id": "acc_1", "app_id": "app_1", "app_name": "Helper",
                "inbox_id": "ib_1", "pod_id": "pod_1", "organization_id": "org_1",
                "first_signed_in_at": "2026-01-01T00:00:00Z",
                "last_signed_in_at": "2026-02-01T00:00:00Z",
                "sign_in_count": 7, "status": "disabled",
                "disabled_at": "2026-02-02T00:00:00Z"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/accounts/acc_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account_id": "acc_1", "app_id": "app_1", "inbox_id": "ib_1",
            "pod_id": "pod_1", "organization_id": "org_1",
            "first_signed_in_at": "2026-01-01T00:00:00Z",
            "last_signed_in_at": "2026-02-01T00:00:00Z",
            "sign_in_count": 7
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/accounts/acc_1/update"))
        .and(body_json(serde_json::json!({ "status": "enabled" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "account_id": "acc_1", "app_id": "app_1", "inbox_id": "ib_1",
            "pod_id": "pod_1", "organization_id": "org_1",
            "first_signed_in_at": "2026-01-01T00:00:00Z",
            "last_signed_in_at": "2026-02-01T00:00:00Z",
            "sign_in_count": 7
        })))
        .expect(1)
        .mount(&server)
        .await;

    let list = client
        .list_accounts(agentmail::AccountsListFilters {
            ascending: Some(true),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(list.accounts[0].app_name.as_deref(), Some("Helper"));
    assert_eq!(list.accounts[0].status.as_deref(), Some("disabled"));

    let account = client.get_account("acc_1").await.unwrap();
    assert_eq!(account.sign_in_count, 7);

    let enabled = client
        .update_account(
            "acc_1",
            agentmail::UpdateAccount {
                status: Some("enabled".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(enabled.account_id, "acc_1");
}

#[tokio::test]
async fn list_accounts_through_inbox_and_pod_scopes() {
    let (server, client) = client().await;
    for p in ["/v0/inboxes/ib_1/accounts", "/v0/pods/pod_1/accounts"] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1, "limit": 50,
                "accounts": [{
                    "account_id": "acc_2", "app_id": "app_1", "inbox_id": "ib_1",
                    "pod_id": "pod_1", "organization_id": "org_1",
                    "first_signed_in_at": "2026-01-01T00:00:00Z",
                    "last_signed_in_at": "2026-02-01T00:00:00Z",
                    "sign_in_count": 2
                }]
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("{p}/acc_2")))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "account_id": "acc_2", "app_id": "app_1", "inbox_id": "ib_1",
                "pod_id": "pod_1", "organization_id": "org_1",
                "first_signed_in_at": "2026-01-01T00:00:00Z",
                "last_signed_in_at": "2026-02-01T00:00:00Z",
                "sign_in_count": 2
            })))
            .expect(1)
            .mount(&server)
            .await;
    }

    for listed in [
        client.inbox("ib_1").list_accounts(Default::default()).await,
        client.pod("pod_1").list_accounts(Default::default()).await,
    ] {
        let list = listed.unwrap();
        assert_eq!(list.count, 1);
        assert_eq!(list.accounts[0].account_id, "acc_2");
    }
    let account = client.inbox("ib_1").get_account("acc_2").await.unwrap();
    assert_eq!(account.sign_in_count, 2);
    let pod_account = client.pod("pod_1").get_account("acc_2").await.unwrap();
    assert_eq!(pod_account.account_id, "acc_2");
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
            human_email: Some("me@example.com".into()),
            username: "my-agent".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(signup.api_key, "am_new");
    assert!(client.agent_verify("123456").await.unwrap().verified);
}
