//! Wire shapes from the AgentMail OpenAPI spec. Every type deserializes
//! permissively (unknown fields ignored, optional fields default) so spec
//! additions don't break callers; request types skip empty fields so the
//! API's validators stay quiet.

use crate::util::QueryBuilder;

mod attachments;
mod drafts;
mod inboxes;
mod messages;
mod webhooks;

pub use attachments::*;
pub use drafts::*;
pub use inboxes::*;
pub use messages::*;
pub use webhooks::*;

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
    pub(crate) fn query(&self) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .opt("limit", self.limit.as_ref())
            .opt("page_token", self.page_token.as_ref())
            .build()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_shapes_round_trip() {
        // Response example from the OpenAPI spec.
        let inbox: Inbox = serde_json::from_str(
            r#"{"pod_id":"pod_1","inbox_id":"ib_1","email":"x@agentmail.to",
                "display_name":"X","updated_at":"2024-01-15T09:30:00Z",
                "created_at":"2024-01-15T09:30:00Z","surprise_field":1}"#,
        )
        .unwrap();
        assert_eq!(inbox.email, "x@agentmail.to");

        // List items are a subset of the get shape, both must parse.
        let list: MessageList = serde_json::from_str(
            r#"{"count":1,"messages":[{"message_id":"m1","thread_id":"t1",
                "from":"a@b.c","subject":"hi","preview":"…","timestamp":"2026-01-01T00:00:00Z"}]}"#,
        )
        .unwrap();
        assert_eq!(list.messages[0].message_id, "m1");
        assert!(list.messages[0].text.is_none());

        // Requests omit empties so the API's validators stay quiet.
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

    #[test]
    fn page_query_pairs() {
        assert!(Page::default().query().is_empty());
        let q = Page {
            limit: Some(10),
            page_token: Some("tok".into()),
        }
        .query();
        assert_eq!(
            q,
            vec![
                ("limit", "10".to_string()),
                ("page_token", "tok".to_string())
            ],
        );
    }
}
