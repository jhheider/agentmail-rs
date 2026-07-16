mod domains;
mod inboxes;
mod messages;
mod webhooks;

pub use domains::*;
pub use inboxes::*;
pub use messages::*;
pub use webhooks::*;

// ─── Page ─────────────────────────────────────────────────────────────────

/// Pagination controls for the `list_*_page` calls. `Default` is the API's
/// own defaults (first page, server-chosen page size).
#[derive(Clone, Debug, Default)]
pub struct Page {
    /// Maximum items per page (the API caps this server-side).
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next_page_token`.
    pub page_token: Option<String>,
}

impl Page {
    /// Build query pairs for appending to a URL.
    pub fn query(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(limit) = self.limit {
            q.push(("limit", limit.to_string()));
        }
        if let Some(token) = &self.page_token {
            q.push(("page_token", token.clone()));
        }
        q
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shapes_round_trip() {
        let inbox: Inbox = serde_json::from_str(
            r#"{"pod_id":"pod_1","inbox_id":"ib_1","email":"x@agentmail.to",
                "display_name":"X","updated_at":"2024-01-15T09:30:00Z",
                "created_at":"2024-01-15T09:30:00Z","surprise_field":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.email, "x@agentmail.to");

        let list: MessageList = serde_json::from_str(
            r#"{"count":1,"messages":[{"message_id":"m1","thread_id":"t1",
                "from":"a@b.c","subject":"hi","preview":"…","timestamp":"2026-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        assert_eq!(list.messages[0].message_id, "m1");
        assert!(list.messages[0].text.is_none());

        let body = serde_json::to_value(SendMessage {
            to: vec!["a@b.c".into()],
            subject: Some("s".into()),
            text: Some("t".into()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({"to":["a@b.c"],"subject":"s","text":"t"}),
        );
    }
}
