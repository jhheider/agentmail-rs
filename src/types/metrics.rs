use std::collections::BTreeMap;

use crate::util::QueryBuilder;
use serde::Deserialize;

/// One time-bucket of an event metric.
#[derive(Clone, Debug, Deserialize)]
pub struct MetricBucket {
    /// Start of the bucket (RFC 3339).
    pub timestamp: String,
    /// Event count in the bucket.
    pub count: i64,
}

/// One time-bucket of a usage metric.
#[derive(Clone, Debug, Deserialize)]
pub struct UsagePoint {
    /// Start of the bucket (RFC 3339).
    pub timestamp: String,
    /// Usage value in the bucket.
    pub value: i64,
}

/// One time-bucket of a rate metric (bounce/complaint percentages).
#[derive(Clone, Debug, Deserialize)]
pub struct RatePoint {
    /// Start of the bucket (RFC 3339).
    pub timestamp: String,
    /// The rate (proportion) observed in the bucket.
    #[serde(default)]
    pub rate: Option<f64>,
    /// Messages sent in the bucket the rate is computed over.
    #[serde(default)]
    pub sent: Option<u64>,
}

/// Event metrics keyed by event type (e.g. `message.received`).
pub type MetricsEvents = BTreeMap<String, Vec<MetricBucket>>;

/// Usage metrics keyed by usage type.
pub type MetricsUsage = BTreeMap<String, Vec<UsagePoint>>;

/// Rate metrics keyed by rate type (`bounce`, `complaint`).
pub type MetricsRates = BTreeMap<String, Vec<RatePoint>>;

/// Query parameters for the metrics endpoints (`get_metrics_events`,
/// `get_metrics_usage`, `get_metrics_rates`). `types` filters by
/// event/usage/rate type; leave it empty for all. `window` is only meaningful
/// for rates (the bucket the rate is computed over, in seconds).
#[derive(Clone, Debug, Default)]
pub struct MetricsQuery {
    /// Event, usage, or rate types to include; empty means all.
    pub types: Vec<String>,
    /// Window start (RFC 3339).
    pub start: Option<String>,
    /// Window end (RFC 3339).
    pub end: Option<String>,
    /// Bucket size in seconds (1 to 86400).
    pub period: Option<u32>,
    /// Rate window in seconds; rates only.
    pub window: Option<u32>,
    /// Maximum buckets to return.
    pub limit: Option<u32>,
    /// Return newest bucket first.
    pub descending: Option<bool>,
}

impl MetricsQuery {
    pub(crate) fn query(&self, types_key: &'static str) -> Vec<(&'static str, String)> {
        QueryBuilder::new()
            .many(types_key, &self.types)
            .opt("start", self.start.as_ref())
            .opt("end", self.end.as_ref())
            .opt("period", self.period.as_ref())
            .opt("window", self.window.as_ref())
            .opt("limit", self.limit.as_ref())
            .opt("descending", self.descending.as_ref())
            .build()
    }
}
