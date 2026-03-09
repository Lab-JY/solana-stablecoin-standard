use axum::{http::StatusCode, response::IntoResponse};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct Metrics {
    inner: Arc<MetricsInner>,
}

struct MetricsInner {
    total_requests: AtomicU64,
    total_errors: AtomicU64,
    request_duration_sum_ms: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(MetricsInner {
                total_requests: AtomicU64::new(0),
                total_errors: AtomicU64::new(0),
                request_duration_sum_ms: AtomicU64::new(0),
            }),
        }
    }

    pub fn inc_requests(&self) {
        self.inner.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_errors(&self) {
        self.inner.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn observe_duration_ms(&self, ms: u64) {
        self.inner
            .request_duration_sum_ms
            .fetch_add(ms, Ordering::Relaxed);
    }

    pub fn total_requests(&self) -> u64 {
        self.inner.total_requests.load(Ordering::Relaxed)
    }

    pub fn total_errors(&self) -> u64 {
        self.inner.total_errors.load(Ordering::Relaxed)
    }

    pub fn request_duration_sum_ms(&self) -> u64 {
        self.inner.request_duration_sum_ms.load(Ordering::Relaxed)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler that serves metrics in Prometheus text exposition format.
pub async fn metrics_handler(
    axum::extract::State(metrics): axum::extract::State<Metrics>,
) -> impl IntoResponse {
    let body = format!(
        "# HELP sss_total_requests Total number of HTTP requests.\n\
         # TYPE sss_total_requests counter\n\
         sss_total_requests {}\n\
         # HELP sss_total_errors Total number of error responses.\n\
         # TYPE sss_total_errors counter\n\
         sss_total_errors {}\n\
         # HELP sss_request_duration_sum_ms Cumulative request duration in milliseconds.\n\
         # TYPE sss_request_duration_sum_ms counter\n\
         sss_request_duration_sum_ms {}\n",
        metrics.total_requests(),
        metrics.total_errors(),
        metrics.request_duration_sum_ms(),
    );

    (StatusCode::OK, [("content-type", "text/plain; version=0.0.4")], body)
}
