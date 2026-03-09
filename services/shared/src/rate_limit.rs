use axum::{
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

struct Bucket {
    tokens: u64,
    last_refill: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, Bucket>>,
    max_requests: u64,
    window_secs: u64,
}

impl RateLimiter {
    pub fn from_env() -> Self {
        let max_requests = std::env::var("RATE_LIMIT_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100);

        let window_secs = std::env::var("RATE_LIMIT_WINDOW_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            buckets: Arc::new(DashMap::new()),
            max_requests,
            window_secs,
        }
    }

    /// Remove entries that have been idle longer than the window.
    pub fn cleanup_expired(&self) {
        let cutoff = Instant::now() - std::time::Duration::from_secs(self.window_secs * 2);
        self.buckets.retain(|_, bucket| bucket.last_refill > cutoff);
    }

    /// Spawn a background task that runs `cleanup_expired()` every 120 seconds.
    ///
    /// Services should call this once during startup to prevent unbounded
    /// memory growth from stale rate-limit buckets.
    pub fn spawn_cleanup_task(&self) {
        let limiter = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(120));
            loop {
                interval.tick().await;
                limiter.cleanup_expired();
            }
        });
    }

    fn check(&self, key: &str) -> Result<(), u64> {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let mut entry = self.buckets.entry(key.to_owned()).or_insert(Bucket {
            tokens: self.max_requests,
            last_refill: now,
        });

        let bucket = entry.value_mut();

        // Refill tokens if the window has elapsed
        if now.duration_since(bucket.last_refill) >= window {
            bucket.tokens = self.max_requests;
            bucket.last_refill = now;
        }

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            let retry_after = window
                .saturating_sub(now.duration_since(bucket.last_refill))
                .as_secs();
            Err(retry_after)
        }
    }
}

/// Rate-limiting middleware keyed by client IP.
///
/// Uses `ConnectInfo<SocketAddr>` when available, otherwise falls back to
/// the `X-Forwarded-For` header or a default key.
pub async fn rate_limit_middleware(
    State(limiter): State<RateLimiter>,
    req: Request,
    next: Next,
) -> Response {
    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.split(',').next().unwrap_or("unknown").trim().to_owned())
        })
        .unwrap_or_else(|| "unknown".to_string());

    match limiter.check(&ip) {
        Ok(()) => next.run(req).await,
        Err(retry_after) => {
            let body = json!({
                "error": "Too many requests",
                "status": 429,
            });
            let mut response = (StatusCode::TOO_MANY_REQUESTS, Json(body)).into_response();
            response.headers_mut().insert(
                "retry-after",
                retry_after.to_string().parse().expect("valid header value"),
            );
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_limiter(max_requests: u64, window_secs: u64) -> RateLimiter {
        RateLimiter {
            buckets: Arc::new(DashMap::new()),
            max_requests,
            window_secs,
        }
    }

    #[test]
    fn requests_within_limit_pass() {
        let limiter = make_limiter(3, 60);
        assert!(limiter.check("ip1").is_ok());
        assert!(limiter.check("ip1").is_ok());
        assert!(limiter.check("ip1").is_ok());
    }

    #[test]
    fn requests_exceeding_limit_rejected() {
        let limiter = make_limiter(2, 60);
        assert!(limiter.check("ip1").is_ok());
        assert!(limiter.check("ip1").is_ok());
        // Third request should fail
        assert!(limiter.check("ip1").is_err());
    }

    #[test]
    fn different_ips_have_separate_buckets() {
        let limiter = make_limiter(1, 60);
        assert!(limiter.check("ip_a").is_ok());
        assert!(limiter.check("ip_b").is_ok());
        // ip_a is exhausted, ip_b is exhausted, but they are independent
        assert!(limiter.check("ip_a").is_err());
        assert!(limiter.check("ip_b").is_err());
    }

    #[test]
    fn cleanup_expired_removes_old_entries() {
        // Window of 0 seconds means entries expire immediately
        let limiter = make_limiter(10, 0);
        limiter.check("old_ip").ok();
        assert_eq!(limiter.buckets.len(), 1);

        // With window_secs=0, cutoff = now - 0*2 = now, so any entry with
        // last_refill <= now should be removed.
        // We need to wait a tiny bit so the entry is older than the cutoff.
        std::thread::sleep(std::time::Duration::from_millis(5));
        limiter.cleanup_expired();
        assert_eq!(limiter.buckets.len(), 0);
    }

    #[test]
    fn err_returns_retry_after_value() {
        let limiter = make_limiter(1, 60);
        limiter.check("ip1").ok();
        let err = limiter.check("ip1").unwrap_err();
        // retry_after should be <= window_secs
        assert!(err <= 60);
    }

    #[test]
    fn zero_limit_rejects_immediately() {
        let limiter = make_limiter(0, 60);
        assert!(limiter.check("ip1").is_err());
    }
}
