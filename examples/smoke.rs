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
    let existing = client.list_inboxes().await?;
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
        .send_message(
            &sender.inbox_id,
            agentmail::SendMessage {
                to: vec![recipient.email.clone()],
                subject: Some("agentmail smoke test".into()),
                text: Some("Sent from an agent's own inbox.".into()),
                ..Default::default()
            },
        )
        .await?;
    println!(
        "sent message {} (thread {})",
        sent.message_id, sent.thread_id
    );

    // Delivery between agentmail.to inboxes is quick but not instant.
    for attempt in 1..=10 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let msgs = client.list_messages(&recipient.inbox_id).await?;
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
