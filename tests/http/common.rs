//! Shared helpers and re-exported wiremock imports for the http test suite.

pub use wiremock::matchers::{body_json, header, method, path, query_param};
pub use wiremock::{Mock, MockServer, ResponseTemplate};

/// A `Client` pointed at a fresh mock server.
pub async fn client() -> (MockServer, agentmail::Client) {
    let server = MockServer::start().await;
    let client = agentmail::Client::new("test-key", server.uri());
    (server, client)
}
