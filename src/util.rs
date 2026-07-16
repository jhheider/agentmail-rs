/// Minimal percent-encoding for path segments (ids are URL-safe in practice;
/// this keeps a stray space, slash, or non-ASCII char from corrupting the
/// path). Encodes per UTF-8 byte, as percent-encoding requires.
pub(crate) fn urlish(segment: &str) -> String {
    segment
        .bytes()
        .map(|b| match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'@' | b'+' => {
                (b as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_segments_stay_paths() {
        assert_eq!(urlish("ib_abc-123.x@y+z"), "ib_abc-123.x@y+z");
        assert_eq!(urlish("a/b c"), "a%2Fb%20c");
        assert_eq!(urlish("café"), "caf%C3%A9");
        assert_eq!(urlish("😀"), "%F0%9F%98%80");
    }

    #[test]
    fn page_query_pairs() {
        use crate::Page;
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
