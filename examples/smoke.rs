//! Live smoke against the real API (needs AGENTMAIL_API_KEY):
//!   cargo run --example smoke
//!
//! Idempotent-ish: reuses inboxes by name if they already exist. Creates two
//! test inboxes (free plan allows 3), sends one message between them, and
//! reads it back.

#[tokio::main]
async fn main() -> Result<(), agentmail::Error> {
    let client = agentmail::Client::from_env()?;

    // Find-or-create the two test inboxes.
    let existing = client.org().list_inboxes(Default::default()).await?;
    println!("existing inboxes: {}", existing.count);
    let find = |username: &str| {
        existing
            .inboxes
            .iter()
            .find(|i| i.email.starts_with(&format!("{username}@")))
            .cloned()
    };
    let mut ensure = Vec::new();
    for (username, display) in [
        ("smoke-sender", "Smoke Sender"),
        ("smoke-recipient", "Smoke Recipient"),
    ] {
        let inbox = match find(username) {
            Some(i) => {
                println!("reusing {}", i.email);
                i
            }
            None => {
                let i = client
                    .org()
                    .create_inbox(agentmail::CreateInbox {
                        username: Some(username.into()),
                        display_name: Some(display.into()),
                        ..Default::default()
                    })
                    .await?;
                println!("created {}", i.email);
                i
            }
        };
        ensure.push(inbox);
    }

    // The sender inbox writes to the recipient inbox; then read it back.
    let (sender, recipient) = (&ensure[0], &ensure[1]);
    let sent = client
        .inbox(&sender.inbox_id)
        .send_text(
            &recipient.email,
            "agentmail smoke test",
            "Sent from an agent's own inbox.",
        )
        .await?;
    println!(
        "sent message {} (thread {})",
        sent.message_id, sent.thread_id
    );

    // Threads: the send created one on the sender side.
    let threads = client
        .inbox(&sender.inbox_id)
        .list_threads(Default::default())
        .await?;
    println!("sender threads: {}", threads.count);

    // Drafts: create, read back, and delete (round-trip without a second send).
    let draft = client
        .inbox(&sender.inbox_id)
        .create_draft(agentmail::CreateDraft {
            to: vec![recipient.email.clone()],
            subject: Some("agentmail smoke draft".into()),
            text: Some("A draft, not sent.".into()),
            ..Default::default()
        })
        .await?;
    let fetched = client
        .inbox(&sender.inbox_id)
        .get_draft(&draft.draft_id)
        .await?;
    println!("draft {} created and fetched", fetched.draft_id);
    client
        .inbox(&sender.inbox_id)
        .delete_draft(&draft.draft_id)
        .await?;
    println!("draft deleted");

    // Delivery between agentmail.to inboxes is quick but not instant.
    for attempt in 1..=10 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let msgs = client
            .inbox(&recipient.inbox_id)
            .list_messages(Default::default())
            .await?;
        if let Some(m) = msgs
            .messages
            .iter()
            .find(|m| m.subject.as_deref() == Some("agentmail smoke test"))
        {
            println!(
                "received in {}: {:?} from {:?}",
                recipient.email, m.subject, m.from
            );
            println!("smoke: OK");
            return Ok(());
        }
        println!("waiting for delivery… ({attempt}/10)");
    }
    println!("smoke: sent OK, but delivery didn't show within 20s (check the dashboard)");
    Ok(())
}
