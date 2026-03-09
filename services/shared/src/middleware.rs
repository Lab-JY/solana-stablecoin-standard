use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

/// Middleware that generates a UUID v4 request ID, attaches it as an
/// `X-Request-Id` response header, and logs request method, path,
/// status code, and duration.
pub async fn request_id_middleware(req: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();

    let mut response = next.run(req).await;

    let duration = start.elapsed();
    let status = response.status();

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
