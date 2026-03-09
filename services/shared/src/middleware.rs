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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, middleware, routing::get, Extension, Router};
    use http::Request;
    use tower::ServiceExt;

    fn obs_app() -> Router {
        let metrics = crate::metrics::Metrics::new();
        Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(middleware::from_fn(observability_middleware))
            .layer(Extension(metrics))
    }

    fn sec_app() -> Router {
        Router::new()
            .route("/ping", get(|| async { "pong" }))
            .layer(middleware::from_fn(security_headers_middleware))
    }

    // --- observability_middleware tests ---

    #[tokio::test]
    async fn response_contains_x_request_id_header() {
        let resp = obs_app()
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(resp.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn request_id_is_valid_uuid() {
        let resp = obs_app()
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let id = resp.headers().get("x-request-id").unwrap().to_str().unwrap();
        assert!(uuid::Uuid::parse_str(id).is_ok(), "not a valid UUID: {id}");
    }

    #[tokio::test]
    async fn each_request_gets_unique_id() {
        let app = obs_app();

        let resp1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let resp2 = app
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let id1 = resp1.headers().get("x-request-id").unwrap().to_str().unwrap().to_owned();
        let id2 = resp2.headers().get("x-request-id").unwrap().to_str().unwrap().to_owned();
        assert_ne!(id1, id2);
    }

    // --- security_headers_middleware tests ---

    #[tokio::test]
    async fn security_headers_present() {
        let resp = sec_app()
            .oneshot(
                Request::builder()
                    .uri("/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let headers = resp.headers();
        assert_eq!(
            headers.get("strict-transport-security").unwrap(),
            "max-age=63072000; includeSubDomains"
        );
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
        assert_eq!(headers.get("cache-control").unwrap(), "no-store");
        assert_eq!(headers.get("referrer-policy").unwrap(), "no-referrer");
    }
}
