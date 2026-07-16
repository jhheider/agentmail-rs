use crate::client::NoBody;
use crate::client::scope::{Metrics, Scoped};
use crate::{Error, types::*};

impl<S: Metrics> Scoped<'_, S> {
    /// GET `{scope}/metrics/events`, event counts bucketed over time, keyed by
    /// event type.
    pub async fn get_metrics_events(&self, query: MetricsQuery) -> Result<MetricsEvents, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/metrics/events", self.base()),
                &query.query("event_types"),
                None::<&NoBody>,
            )
            .await
    }

    /// GET `{scope}/metrics/usage`, usage values bucketed over time, keyed by
    /// usage type.
    pub async fn get_metrics_usage(&self, query: MetricsQuery) -> Result<MetricsUsage, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/metrics/usage", self.base()),
                &query.query("usage_types"),
                None::<&NoBody>,
            )
            .await
    }
}
