use crate::common::*;

#[tokio::test]
async fn create_domain_returns_records() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/domains"))
        .and(body_json(serde_json::json!({ "domain": "mail.example.com" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "domain_id": "dom_1",
            "domain": "mail.example.com",
            "status": "pending",
            "feedback_enabled": true,
            "subdomains_enabled": false,
            "records": [
                {"type": "TXT", "name": "mail", "value": "v=spf1 ...", "status": "pending"},
                {"type": "MX", "name": "mail", "value": "feedback-smtp", "status": "pending", "priority": 10}
            ],
            "updated_at": "2026-01-01T00:00:00Z",
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let domain = client
        .org()
        .create_domain(agentmail::CreateDomain {
            domain: "mail.example.com".into(),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(domain.domain_id, "dom_1");
    assert_eq!(domain.records.len(), 2);
    assert_eq!(domain.records[1].record_type, "MX");
    assert_eq!(domain.records[1].priority, Some(10));
}

#[tokio::test]
async fn list_get_update_delete_domain() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/domains"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "domains": [{"domain_id": "dom_1", "domain": "a.com"}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/domains/dom_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "domain_id": "dom_1", "domain": "a.com", "status": "verified"
        })))
        .mount(&server)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/v0/domains/dom_1"))
        .and(body_json(serde_json::json!({ "feedback_enabled": false })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "domain_id": "dom_1"
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v0/domains/dom_1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    assert_eq!(
        client
            .org()
            .list_domains(Default::default())
            .await
            .unwrap()
            .count,
        1
    );
    assert_eq!(
        client
            .org()
            .get_domain("dom_1")
            .await
            .unwrap()
            .status
            .as_deref(),
        Some("verified")
    );
    client
        .org()
        .update_domain(
            "dom_1",
            agentmail::UpdateDomain {
                feedback_enabled: Some(false),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    client.org().delete_domain("dom_1").await.unwrap();
}

#[tokio::test]
async fn verify_domain_and_zone_file() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/domains/dom_1/verify"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/domains/dom_1/zone-file"))
        .respond_with(ResponseTemplate::new(200).set_body_string("mail 300 IN TXT \"v=spf1\"\n"))
        .expect(1)
        .mount(&server)
        .await;

    client.org().verify_domain("dom_1").await.unwrap();
    let zone = client.org().get_domain_zone_file("dom_1").await.unwrap();
    assert!(zone.contains("v=spf1"));
}
