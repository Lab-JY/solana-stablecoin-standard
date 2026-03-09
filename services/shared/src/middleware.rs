use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

use crate::metrics::Metrics;

/// Middleware that generates a UUID v4 request ID, inserts it into request
/// extensions, attaches it as an `X-Request-Id` response header, records
/// metrics (request count, error count, duration), and logs request details.
///
/// Requires `Metrics` to be installed as an axum extension (e.g. via
/// `Extension(metrics)` layer).
pub async fn observability_middleware(
    axum::extract::Extension(metrics): axum::extract::Extension<Metrics>,
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();

    // Q-15: propagate request ID into request extensions so downstream
    // handlers/extractors can access it.
    req.extensions_mut().insert(request_id.clone());

    let mut response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

    // Q-12 / Q-13: record metrics
    metrics.inc_requests();
    if status.as_u16() >= 400 {
        metrics.inc_errors();
    }
    metrics.observe_duration_ms(duration.as_millis() as u64);

    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        status = status.as_u16(),
        duration_ms = duration.as_millis() as u64,
        "request completed"
    );

    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().expect("valid header value"),
    );

    response
}

/// Middleware that adds security headers to every response.
pub async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();
    headers.insert(
        "strict-transport-security",
        "max-age=63072000; includeSubDomains"
            .parse()
            .expect("valid header value"),
    );
    headers.insert(
        "x-content-type-options",
        "nosniff".parse().expect("valid header value"),
    );
    headers.insert(
        "x-frame-options",
        "DENY".parse().expect("valid header value"),
    );
    headers.insert(
        "cache-control",
        "no-store".parse().expect("valid header value"),
    );
    headers.insert(
        "referrer-policy",
        "no-referrer".parse().expect("valid header value"),
    );

    response
}
