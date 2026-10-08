use crate::common::*;

#[tokio::test]
async fn get_metrics_events_keyed_by_type() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/metrics/events"))
        .and(query_param("event_types", "message.received"))
        .and(query_param("period", "3600"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "message.received": [
                {"timestamp": "2026-01-01T00:00:00Z", "count": 5},
                {"timestamp": "2026-01-01T01:00:00Z", "count": 3}
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let events = client
        .org()
        .get_metrics_events(agentmail::MetricsQuery {
            types: vec!["message.received".into()],
            period: Some(3600),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(events["message.received"].len(), 2);
    assert_eq!(events["message.received"][0].count, 5);
}

#[tokio::test]
async fn get_metrics_usage_keyed_by_type() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/metrics/usage"))
        .and(query_param("usage_types", "storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "storage": [{"timestamp": "2026-01-01T00:00:00Z", "value": 1024}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let usage = client
        .org()
        .get_metrics_usage(agentmail::MetricsQuery {
            types: vec!["storage".into()],
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(usage["storage"][0].value, 1024);
}

#[tokio::test]
async fn list_inbox_events_paginates() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/events"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "events": [{
                "event_id": "ev_1", "event_type": "label.added",
                "inbox_id": "ib_1", "message_id": "m1", "label": "unread"
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let events = client
        .inbox("ib_1")
        .list_events(Default::default())
        .await
        .unwrap();
    assert_eq!(events.count, 1);
    assert_eq!(events.events[0].event_id, "ev_1");
}

#[tokio::test]
async fn get_metrics_rates_at_all_scopes() {
    let (server, client) = client().await;
    for p in [
        "/v0/metrics/rates",
        "/v0/inboxes/ib_1/metrics/rates",
        "/v0/pods/pod_1/metrics/rates",
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            .and(query_param("rate_types", "bounce"))
            .and(query_param("window", "86400"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bounce": [
                    {"timestamp": "2026-01-01T00:00:00Z", "rate": 0.02, "sent": 100},
                    {"timestamp": "2026-01-02T00:00:00Z", "rate": 0.0, "sent": 50}
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;
    }
    let q = agentmail::MetricsQuery {
        types: vec!["bounce".into()],
        window: Some(86400),
        ..Default::default()
    };
    for scope_rates in [
        client.org().get_metrics_rates(q.clone()).await,
        client.inbox("ib_1").get_metrics_rates(q.clone()).await,
        client.pod("pod_1").get_metrics_rates(q).await,
    ] {
        let rates = scope_rates.unwrap();
        assert_eq!(rates["bounce"].len(), 2);
        assert_eq!(rates["bounce"][0].rate, Some(0.02));
        assert_eq!(rates["bounce"][0].sent, Some(100));
    }
}
