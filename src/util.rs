//! Small internal helpers shared across the resource modules.

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
/// Accumulates query-string parameters, skipping absent ones. Shared by
/// [`Page`] and the per-resource filter builders so the list endpoints all
/// serialize their parameters the same way.
pub(crate) struct QueryBuilder(Vec<(&'static str, String)>);
impl QueryBuilder {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// Push `key=value` when `value` is `Some`; skip it otherwise.
    pub(crate) fn opt(mut self, key: &'static str, value: Option<&impl ToString>) -> Self {
        if let Some(value) = value {
            self.0.push((key, value.to_string()));
        }
        self
    }

    /// Push one `key=value` pair per element (repeated-key array parameters).
    pub(crate) fn many(mut self, key: &'static str, values: &[String]) -> Self {
        for value in values {
            self.0.push((key, value.clone()));
        }
        self
    }

    /// Finish and return the accumulated parameters.
    pub(crate) fn build(self) -> Vec<(&'static str, String)> {
        self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_segments_stay_paths() {
        assert_eq!(urlish("ib_abc-123.x@y+z"), "ib_abc-123.x@y+z");
        assert_eq!(urlish("a/b c"), "a%2Fb%20c");
        // Non-ASCII encodes per UTF-8 byte, not per code point.
        assert_eq!(urlish("café"), "caf%C3%A9");
        assert_eq!(urlish("😀"), "%F0%9F%98%80");
    }
}
