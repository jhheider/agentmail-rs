use crate::common::*;
use agentmail::{ListDirection, ListKind};

#[tokio::test]
async fn create_and_list_block_entries() {
    let (server, client) = client().await;
    Mock::given(method("POST"))
        .and(path("/v0/lists/receive/block"))
        .and(body_json(
            serde_json::json!({ "entry": "spam.example", "reason": "phishing" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "entry": "spam.example", "reason": "phishing",
            "entry_type": "domain", "read_only": false,
            "created_at": "2026-01-01T00:00:00Z"
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v0/lists/receive/block"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "count": 1,
            "entries": [{"entry": "spam.example", "entry_type": "domain", "read_only": false}]
        })))
        .mount(&server)
        .await;

    let created = client
        .org()
        .create_list_entry(
            ListDirection::Receive,
            ListKind::Block,
            agentmail::CreateListEntry {
                entry: "spam.example".into(),
                reason: Some("phishing".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(created.entry_type, agentmail::EntryType::Domain);

    let page = client
        .org()
        .list_entries(ListDirection::Receive, ListKind::Block, Default::default())
        .await
        .unwrap();
    assert_eq!(page.count, 1);
}

#[tokio::test]
async fn get_and_delete_entry() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/lists/send/allow/a@b.c"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "entry": "a@b.c", "entry_type": "email", "read_only": false
        })))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v0/lists/send/allow/a@b.c"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let entry = client
        .org()
        .get_list_entry(ListDirection::Send, ListKind::Allow, "a@b.c")
        .await
        .unwrap();
    assert_eq!(entry.entry, "a@b.c");
    client
        .org()
        .delete_list_entry(ListDirection::Send, ListKind::Allow, "a@b.c")
        .await
        .unwrap();
}
