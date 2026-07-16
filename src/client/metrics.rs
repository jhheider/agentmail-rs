use crate::client::NoBody;
use crate::{Client, Error, types::*};

impl Client {
    /// GET /v0/metrics/events, event counts bucketed over time, keyed by event
    /// type.
    pub async fn get_metrics_events(&self, query: MetricsQuery) -> Result<MetricsEvents, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/metrics/events",
            &query.query("event_types"),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/metrics/usage, usage values bucketed over time, keyed by usage
    /// type.
    pub async fn get_metrics_usage(&self, query: MetricsQuery) -> Result<MetricsUsage, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/metrics/usage",
            &query.query("usage_types"),
            None::<&NoBody>,
        )
        .await
    }
}
