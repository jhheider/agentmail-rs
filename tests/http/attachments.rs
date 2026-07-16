use crate::common::*;

#[tokio::test]
async fn get_message_attachment_returns_download_url() {
    let (server, client) = client().await;
    Mock::given(method("GET"))
        .and(path("/v0/inboxes/ib_1/messages/m1/attachments/at_1"))
        .and(header("authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "attachment_id": "at_1",
            "filename": "invoice.pdf",
            "size": 1234,
            "content_type": "application/pdf",
            "download_url": "https://s3.example/at_1?sig=abc",
            "expires_at": "2026-01-01T00:00:00Z"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let att = client
        .inbox("ib_1")
        .get_message_attachment("m1", "at_1")
        .await
        .unwrap();
    assert_eq!(att.attachment_id, "at_1");
    assert_eq!(att.size, Some(1234));
    assert!(att.download_url.is_some());
}

#[tokio::test]
async fn get_thread_and_draft_attachment_paths() {
    let (server, client) = client().await;
    for p in [
        "/v0/inboxes/ib_1/threads/th_1/attachments/at_1",
        "/v0/inboxes/ib_1/drafts/dr_1/attachments/at_1",
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "attachment_id": "at_1", "size": 1
            })))
            .expect(1)
            .mount(&server)
            .await;
    }
    client
        .inbox("ib_1")
        .get_thread_attachment("th_1", "at_1")
        .await
        .unwrap();
    client
        .inbox("ib_1")
        .get_draft_attachment("dr_1", "at_1")
        .await
        .unwrap();
}

#[tokio::test]
async fn download_attachment_fetches_presigned_url_without_auth() {
    let (server, client) = client().await;
    // The presigned URL is served by the same mock server, at a path with no
    // Authorization requirement; download_attachment must not send the bearer.
    Mock::given(method("GET"))
        .and(path("/download/at_1"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"PDFBYTES".to_vec()))
        .expect(1)
        .mount(&server)
        .await;

    let att = agentmail::Attachment {
        attachment_id: "at_1".into(),
        download_url: Some(format!("{}/download/at_1", server.uri())),
        ..att_stub()
    };
    let bytes = client.download_attachment(&att).await.unwrap();
    assert_eq!(bytes, b"PDFBYTES");
}

#[tokio::test]
async fn download_attachment_without_url_errors() {
    let (_server, client) = client().await;
    let att = att_stub();
    match client.download_attachment(&att).await {
        Err(agentmail::Error::NoDownloadUrl) => {}
        other => panic!("expected NoDownloadUrl, got {other:?}"),
    }
}

/// A minimal `Attachment` with only the required id set.
fn att_stub() -> agentmail::Attachment {
    serde_json::from_value(serde_json::json!({"attachment_id": "at_1"})).unwrap()
}
