use crate::common::*;

#[tokio::test]
async fn create_and_get_pod() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/pods"))
        .and(body_json(serde_json::json!({ "name": "team-a" })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pod_id": "pod_1", "name": "team-a",
            "updated_at": "2026-01-01T00:00:00Z", "created_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/pods/pod_1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pod_id": "pod_1", "name": "team-a"
        })))
        .mount(&server)
        .await;

    let pod = client
        .create_pod(agentmail::CreatePod {
            name: Some("team-a".into()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(pod.pod_id, "pod_1");
    assert_eq!(
        client.get_pod("pod_1").await.unwrap().name.as_deref(),
        Some("team-a")
    );
}

#[tokio::test]
async fn list_and_delete_pod() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/pods"))
        .and(query_param("limit", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1, "pods": [{"pod_id": "pod_1", "name": "team-a"}],
            "next_page_token": null
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v0/pods/pod_1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let list = client
        .list_pods_page(agentmail::Page {
            limit: Some(2),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(list.count, 1);
    client.delete_pod("pod_1").await.unwrap();
}
