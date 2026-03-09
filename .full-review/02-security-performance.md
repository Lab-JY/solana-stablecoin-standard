# Phase 2: Security & Performance Review

## Security Findings (22 new findings)

### Critical (4)

| ID | CWE | File | Issue |
|----|-----|------|-------|
| P2-01 | CWE-306 | listener.rs:169-196 | Indexer-to-webhook has no authentication (no Bearer token sent) |
| P2-02 | CWE-20 | mint-burn/handlers.rs:33-35 | No mint amount upper bound (u64::MAX possible) |
| P2-03 | CWE-312 | solana.rs:25-29 | Private key material not zeroed from memory after loading |
| P2-04 | CWE-798 | docker-compose.yml:20,51,80,104 | Second hardcoded default secret "sss-dev-secret-key" |

### High (6)

| ID | CWE | File | Issue |
|----|-----|------|-------|
| P2-05 | CWE-290 | rate_limit.rs:89-99 | X-Forwarded-For spoofing bypasses rate limiter |
| P2-06 | CWE-770 | rate_limit.rs:21,39,55 | Rate-limit DashMap grows unboundedly (cleanup never called) |
| P2-07 | CWE-362 | docker-compose.yml, db.rs | Shared SQLite no busy_timeout configured |
| P2-08 | CWE-942 | All main.rs | Wildcard CORS default with auth headers |
| P2-09 | CWE-400 | listener.rs:175 | New HTTP client created per event forward |
| P2-10 | CWE-693 | All main.rs | Missing security headers (HSTS, X-Content-Type-Options, etc.) |

### Medium (7)

| ID | CWE | File | Issue |
|----|-----|------|-------|
| P2-11 | CWE-354 | db.rs:50-67 | Audit log has no tamper protection |
| P2-12 | CWE-209 | error.rs:67-70 | Database errors leak schema details |
| P2-13 | CWE-20 | types.rs:115-118 | Negative pagination values bypass limits |
| P2-14 | CWE-89 | compliance/handlers.rs:144-183 | Unvalidated datetime in dynamic SQL |
| P2-15 | CWE-1104 | Dockerfile:43-48 | curl in runtime image aids lateral movement |
| P2-16 | CWE-400 | All main.rs | No explicit request body size limit |
| P2-17 | CWE-400 | delivery.rs:36-46 | Webhook delivery has no concurrency cap |

### Low (5)

| ID | CWE | File | Issue |
|----|-----|------|-------|
| P2-18 | CWE-200 | auth.rs:39 | Metrics endpoint exposed without auth |
| P2-19 | CWE-778 | webhook/handlers.rs:104-128 | Webhook deletion not audit-logged |
| P2-20 | CWE-295 | delivery.rs:16-18 | No explicit TLS enforcement on outbound |
| P2-21 | CWE-863 | compliance/handlers.rs:77-106 | No two-person control on blacklist removal |
| P2-22 | CWE-362 | metrics.rs:28-29 | Relaxed atomic ordering may produce stale reads |

## Performance Findings (from Phase 1 + additional)

### High

| ID | File | Issue |
|----|------|-------|
| Q-05 | solana.rs:44-49 | Blocking Solana RPC calls inside async functions starve tokio runtime |
| Q-06 | rate_limit.rs:46-49 | Rate limiter DashMap never cleaned (memory leak) |
| P2-09 | listener.rs:175 | New reqwest::Client per event (connection pool churn) |

### Medium

| ID | File | Issue |
|----|------|-------|
| Q-14 | docker-compose.yml | Shared SQLite write contention across 4 services |
| P2-07 | db.rs | No busy_timeout on SQLite connection |
| P2-17 | delivery.rs:36-46 | Sequential delivery processing (50 * 10s = 500s worst case) |
| Q-12 | metrics.rs + middleware.rs | Metrics struct never wired into middleware (always zero) |
| Q-08 | mint-burn/handlers.rs:243 | Hardcoded decimals=6 (should query from chain) |
| DB-01 | db.rs:17-19 | Connection pool max=5 may be limiting under load |

### Low

| ID | File | Issue |
|----|------|-------|
| Q-15 | middleware.rs:8-34 | Request ID not propagated to downstream context |
| Q-17 | parser.rs, delivery.rs | hex_encode duplicated (minor alloc overhead) |

## Critical Issues for Phase 3 Context

- P2-01: Inter-service communication has no auth — integration tests needed
- P2-02: No amount validation — fuzz/boundary tests needed
- P2-11: Audit log tamper protection — compliance documentation needed
- Security headers (P2-10) and CORS (P2-08) should be documented in OPERATIONS.md
- Performance issues (Q-05, P2-09) affect production reliability
