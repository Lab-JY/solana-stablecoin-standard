# Comprehensive Code Review Report

## Review Target

Backend service optimization commit `c25acee` — Solana Stablecoin Standard (SSS) backend services (4 Axum microservices + shared library). Added authentication, rate limiting, metrics, event parser, pagination, Docker improvements.

## Executive Summary

The backend services optimization adds meaningful security and observability infrastructure, but the implementation contains **6 Critical security vulnerabilities** that must be fixed before any deployment, including hardcoded default secrets, non-constant-time token comparison, and unauthenticated inter-service communication. Documentation is severely out of sync with the implementation, and the new middleware modules have zero test coverage.

## Findings by Priority

### Critical Issues (P0 — Must Fix Immediately): 10

| ID | Category | Issue | File |
|----|----------|-------|------|
| Q-01 | Security | Default API secret "changeme" in auth.rs | shared/auth.rs:19 |
| Q-02 | Security | Non-constant-time Bearer token comparison | shared/auth.rs:51 |
| P2-01 | Security | Indexer-to-webhook has no authentication | indexer/listener.rs:169 |
| P2-02 | Security | No mint amount upper bound | mint-burn/handlers.rs:33 |
| P2-03 | Security | Private key material not zeroed from memory | shared/solana.rs:25 |
| P2-04 | Security | Second hardcoded docker-compose secret | docker-compose.yml:20 |
| TC-01 | Testing | Zero tests for auth/rate-limit/middleware | shared/src/*.rs |
| TC-08 | Testing | Zero security-specific tests | all services |
| DOC-02 | Docs | API.md documents 6+ non-existent endpoints | docs/API.md |
| DOC-03 | Docs | API.md schemas diverge from actual types | docs/API.md |

### High Priority (P1 — Fix Before Release): 18

| ID | Category | Issue |
|----|----------|-------|
| Q-03 | Quality | 90+ lines duplicated across 3 main.rs files |
| Q-04 | Quality | Health handler duplicated 4 times |
| Q-05 | Perf | Blocking Solana RPC in async functions |
| Q-06 | Perf | Rate limiter cleanup never scheduled (memory leak) |
| Q-07 | Security | Internal error details leaked to responses |
| A-01 | Arch | Shared types violate Interface Segregation |
| A-02 | Arch | No data access layer — raw SQL in handlers |
| A-03 | Arch | Middleware stack order incorrect (rate-limit after auth) |
| A-04 | Arch | Indexer has no health check |
| P2-05 | Security | X-Forwarded-For spoofing bypasses rate limiter |
| P2-08 | Security | Wildcard CORS default with auth headers |
| P2-09 | Perf | New HTTP client per event forward |
| P2-10 | Security | Missing security response headers |
| TC-02 | Testing | Zero tests for HTTP handlers |
| TC-03 | Testing | Zero tests for webhook delivery/HMAC |
| TC-10 | Testing | No backend E2E integration tests |
| DOC-04 | Docs | Auth model incorrectly documented |
| DOC-07 | Docs | OPERATIONS.md omits backend env vars |

### Medium Priority (P2 — Plan for Next Sprint): 23

| ID | Category | Issue |
|----|----------|-------|
| Q-08 | Correctness | Hardcoded decimals=6 |
| Q-09 | Security | Webhook URL allows SSRF |
| Q-10 | Correctness | Non-atomic webhook deletion |
| Q-11 | Correctness | Racy INSERT+SELECT in event receive |
| Q-12 | Observability | Metrics never wired into middleware |
| Q-13 | Observability | Prometheus lacks labels/histogram |
| Q-14 | Arch | Shared SQLite write contention |
| Q-15 | Observability | Request ID not propagated |
| A-05 | Quality | AppState defined redundantly |
| A-06 | Arch | Migrations run from all 4 services |
| A-07 | Reliability | Delivery worker no graceful shutdown |
| A-08 | Config | Indexer receives unused HTTP env vars |
| A-09 | Security | Webhook secrets in plaintext |
| P2-11 | Security | Audit log no tamper protection |
| P2-12 | Security | DB errors leak schema details |
| P2-13 | Validation | Negative pagination bypass |
| P2-14 | Security | Unvalidated datetime in dynamic SQL |
| P2-16 | Security | No request body size limit |
| P2-17 | Perf | Webhook delivery no concurrency cap |
| DOC-05 | Docs | Env var names mismatch |
| DOC-06 | Docs | Webhook retry policy mismatch |
| DOC-08 | Docs | No production deployment guide |
| DOC-12 | Docs | Missing .env.example |

### Low Priority (P3 — Track in Backlog): 8

| ID | Category | Issue |
|----|----------|-------|
| Q-16 | Quality | Repetitive if-chain in parser |
| Q-17 | Quality | hex_encode duplicated |
| Q-18 | Security | X-Forwarded-For trusted without validation |
| A-10 | Config | Deprecated docker-compose version |
| A-11 | Config | No container resource limits |
| P2-18 | Security | Metrics exposed without auth |
| P2-19 | Logging | Webhook deletion not audit-logged |
| DOC-11 | Docs | README missing auth/metrics info |

## Findings by Category

- **Security**: 22 findings (6 Critical, 6 High, 8 Medium, 2 Low)
- **Code Quality**: 7 findings (0 Critical, 3 High, 1 Medium, 3 Low)
- **Architecture**: 9 findings (0 Critical, 4 High, 5 Medium, 0 Low)
- **Performance**: 6 findings (0 Critical, 3 High, 2 Medium, 1 Low)
- **Testing**: 10 findings (2 Critical, 5 High, 3 Medium, 0 Low)
- **Documentation**: 12 findings (2 Critical, 4 High, 4 Medium, 2 Low)

## Recommended Action Plan

### Immediate (before any deployment)
1. **[small]** Q-01: Remove default secret — panic if API_SECRET_KEY unset
2. **[small]** Q-02: Add `subtle` crate, use constant-time comparison
3. **[small]** P2-04: Remove docker-compose default secrets
4. **[small]** P2-02: Add max mint amount validation
5. **[small]** P2-03: Add `zeroize` for keypair buffers
6. **[medium]** P2-01: Add Bearer token to indexer→webhook HTTP calls
7. **[medium]** A-03: Fix middleware stack order (rate-limit before auth)
8. **[medium]** Q-07 + P2-12: Sanitize error messages for clients

### Before competition submission
9. **[medium]** DOC-02 + DOC-03: Fix API.md (remove phantom endpoints, correct schemas)
10. **[medium]** DOC-04 + DOC-07: Fix auth model docs and add env var documentation
11. **[small]** P2-08: Set restrictive CORS default
12. **[small]** P2-10: Add security response headers middleware
13. **[small]** Q-06 + P2-06: Spawn periodic rate limiter cleanup task

### Next sprint
14. **[large]** TC-01 + TC-02: Write unit tests for middleware and handlers
15. **[large]** A-02: Extract data access layer from handlers
16. **[medium]** Q-03 + Q-04: Deduplicate main.rs and health handlers into shared lib
17. **[medium]** DOC-09: Document middleware stack in ARCHITECTURE.md

## Review Metadata

- Review date: 2026-03-10
- Phases completed: Phase 1 (Quality+Architecture), Phase 2 (Security+Performance), Phase 3 (Testing+Documentation)
- Flags: security_focus=yes, framework=Axum
- Total findings: 59
- Critical: 10 | High: 18 | Medium: 23 | Low: 8
