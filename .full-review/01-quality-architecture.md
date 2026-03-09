# Phase 1: Code Quality & Architecture Review

## Summary

- **Critical:** 2 findings
- **High:** 9 findings
- **Medium:** 14 findings
- **Low:** 4 findings

## Critical Issues

### Q-01: Insecure Default API Secret
- **File:** shared/src/auth.rs:19, docker-compose.yml:20
- `AuthState::from_env()` falls back to hardcoded `"changeme"`. Production deployable with trivially guessable secret.
- **Fix:** Panic at startup if `API_SECRET_KEY` not set.

### Q-02: Non-Constant-Time Bearer Token Comparison
- **File:** shared/src/auth.rs:51
- Uses `==` for token comparison, vulnerable to timing side-channel attacks.
- **Fix:** Use `subtle::ConstantTimeEq` for comparison.

## High Priority Issues

### Q-03: 90+ Lines Duplicated Across 3 Service main.rs
- `build_cors_layer()` (21 lines) and `shutdown_signal()` (25 lines) copied verbatim.
- **Fix:** Move to `sss_shared`.

### Q-04: Health Handler Duplicated 4 Times
- Nearly identical `health()` across all services.
- **Fix:** Generic health check in shared library.

### Q-05: Blocking Solana RPC Calls in Async Functions
- **File:** shared/src/solana.rs:44-49
- `send_and_confirm_transaction` is synchronous, blocks tokio runtime.
- **Fix:** Use `tokio::task::spawn_blocking` or nonblocking RPC client.

### Q-06: Rate Limiter Cleanup Never Scheduled
- **File:** shared/src/rate_limit.rs:46-49
- `cleanup_expired()` exists but never called. DashMap grows unbounded.
- **Fix:** Spawn periodic cleanup task.

### Q-07: Internal Error Details Leaked to HTTP Responses
- **File:** shared/src/error.rs:39-58
- Database errors, stack traces exposed to clients.
- **Fix:** Return generic messages for Internal/Database/Solana errors.

### A-01: Shared Types Violate Interface Segregation
- All domain types in single types.rs. Webhook pulls mint/burn types.
- **Fix:** Split into per-domain sub-modules.

### A-02: No Data Access Layer — Raw SQL in Handlers
- SQL scattered across all handlers. Same blacklist query in 3 places.
- **Fix:** Create repository/DAO methods on Database struct.

### A-03: Middleware Stack Order Incorrect
- Rate limiting runs AFTER auth. Brute-force attacks bypass rate limiting.
- **Fix:** Reorder: Rate limit before auth in layer stack.

### A-04: Indexer Has No Health Check
- No HTTP endpoint, no Docker healthcheck.
- **Fix:** Add lightweight /health endpoint or file-based probe.

## Medium Priority Issues

| ID | Issue | File |
|----|-------|------|
| Q-08 | Hardcoded decimals=6 in supply | mint-burn/handlers.rs:243 |
| Q-09 | Webhook URL allows SSRF | webhook/handlers.rs:31-35 |
| Q-10 | Non-atomic webhook deletion | webhook/handlers.rs:110-128 |
| Q-11 | Racy INSERT+SELECT in event receive | webhook/handlers.rs:148-166 |
| Q-12 | Metrics struct never wired into middleware | metrics.rs + middleware.rs |
| Q-13 | Prometheus output lacks labels/histogram | metrics.rs:60-80 |
| Q-14 | Shared SQLite volume write contention | docker-compose.yml |
| Q-15 | Request ID not propagated to downstream | middleware.rs:8-34 |
| A-05 | AppState defined redundantly per service | all main.rs |
| A-06 | Migrations run from all 4 services | db.rs:23 |
| A-07 | Delivery worker has no graceful shutdown | delivery.rs:21-27 |
| A-08 | Indexer receives unused HTTP config env vars | docker-compose.yml |
| A-09 | Webhook secrets stored/returned plaintext | db.rs, webhook/handlers.rs |

## Low Priority Issues

| ID | Issue | File |
|----|-------|------|
| Q-16 | Repetitive if-chain in event parser | parser.rs:81-214 |
| Q-17 | hex_encode implemented twice | parser.rs, delivery.rs |
| Q-18 | X-Forwarded-For trusted without validation | rate_limit.rs:93-98 |
| A-10 | Deprecated docker-compose version field | docker-compose.yml:1 |
| A-11 | No container resource limits | docker-compose.yml |

## Critical Issues for Phase 2 Context

- Q-01 and Q-02 are exploitable security vulnerabilities requiring immediate fix
- A-03 middleware ordering allows auth bypass of rate limiting
- Q-07 error leakage aids attacker reconnaissance
- Q-09 webhook SSRF potential
- A-09 webhook secrets in plaintext
