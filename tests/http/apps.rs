use crate::common::*;

fn app_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "app_id": id, "slug": "helper", "name": "Helper",
        "description": "A helpful app", "categories": ["ai", "productivity"],
        "updated_at": "2026-01-01T00:00:00Z"
    })
}

#[tokio::test]
async fn list_search_and_get_apps() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/apps"))
        .and(query_param("category", "ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "limit": 50, "apps": [app_json("app_1")]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/apps/search"))
        .and(query_param("q", "help"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "limit": 5, "apps": [app_json("app_1")]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/apps/app_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(app_json("app_1")))
        .expect(1)
        .mount(&server)
        .await;

    let list = client
        .list_apps(agentmail::AppsListFilters {
            category: Some("ai".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(list.apps[0].name.as_deref(), Some("Helper"));
    assert_eq!(list.apps[0].categories, vec!["ai", "productivity"]);

    let found = client.search_apps("help", Some(5)).await.unwrap();
    assert_eq!(found.count, 1);

    let app = client.get_app("app_1").await.unwrap();
    assert_eq!(app.slug.as_deref(), Some("helper"));
}

#[tokio::test]
async fn app_accounts_and_connect_flow() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/apps/app_1/accounts"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "app": app_json("app_1"), "count": 1, "limit": 50,
            "accounts": [{
                "account_id": "acc_1", "app_id": "app_1", "app_name": "Helper",
                "inbox_id": "ib_1", "pod_id": "pod_1", "organization_id": "org_1",
                "first_signed_in_at": "2026-01-01T00:00:00Z",
                "last_signed_in_at": "2026-02-01T00:00:00Z",
                "sign_in_count": 3
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v0/apps/app_1/connect"))
        .and(body_json(serde_json::json!({
            "inbox_id": "ib_1", "accept_disclosure": true
        })))
        .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
            "api_key_id": "key_9",
            "magic_url": "https://agentmail.to/connect/abc",
            "expires_at": "2026-03-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let accounts = client
        .list_app_accounts("app_1", Default::default())
        .await
        .unwrap();
    assert_eq!(accounts.app.unwrap().app_id, "app_1");
    assert_eq!(accounts.accounts[0].sign_in_count, 3);

    let connect = client
        .connect_app(
            "app_1",
            agentmail::ConnectApp {
                inbox_id: Some("ib_1".into()),
                accept_disclosure: Some(true),
            },
        )
        .await
        .unwrap();
    assert_eq!(connect.api_key_id, "key_9");
    assert!(connect.magic_url.ends_with("/connect/abc"));
}
