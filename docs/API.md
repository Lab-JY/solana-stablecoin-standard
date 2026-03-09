# API Reference

Backend services REST API documentation for the SSS microservices.

## Overview

The SSS backend consists of four containerized Rust/Axum services:

| Service | External Port | Internal Port | Purpose |
|---------|---------------|---------------|---------|
| mint-burn | 3001 | 3000 | Mint/burn lifecycle management |
| compliance | 3002 | 3000 | Blacklist management, audit trail |
| indexer | (none) | (none) | On-chain event indexing via WebSocket (no HTTP server) |
| webhook | 3004 | 3000 | Webhook registration and event delivery |

Each HTTP service listens internally on port 3000 and is mapped to its external port via Docker Compose. The indexer is a WebSocket-only process with no HTTP endpoints.

All HTTP services share:
- Structured JSON logging via `tracing`
- SQLite database via `sqlx`
- `GET /health` endpoint
- `.env` configuration
- JSON request/response format
- Bearer token authentication (via `API_SECRET_KEY`)
- Rate limiting (configurable via `RATE_LIMIT_MAX` / `RATE_LIMIT_WINDOW_SECS`)
- CORS (configurable via `ALLOWED_ORIGINS`)

## Docker Setup

### Quick Start

```bash
cd services/

# Build and start all services
docker compose up -d

# Check health
curl http://localhost:3001/health
curl http://localhost:3002/health
curl http://localhost:3004/health
```

### Docker Compose

See `services/docker-compose.yml` for the full configuration. Key mappings:

```yaml
services:
  mint-burn:
    ports: ["3001:3000"]
  compliance:
    ports: ["3002:3000"]
  indexer:
    # No ports - WebSocket listener only
  webhook:
    ports: ["3004:3000"]
```

## Authentication

All endpoints require a Bearer token **except** `GET /health` and `GET /metrics`.

```
Authorization: Bearer <API_SECRET_KEY>
```

The token is validated against the `API_SECRET_KEY` environment variable using constant-time comparison.

**Unauthorized response (401):**

```json
{
  "error": "Missing authorization header",
  "status": 401
}
```

## Mint-Burn Service (Port 3001)

### POST /mint

Mint tokens to a recipient address.

**Request:**

```json
{
  "recipient": "7Xf3kP9QwR2mN8...",
  "amount": 1000000000,
  "mint": "4Gh2mN8qW1tR5..."
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `recipient` | string | yes | Solana public key of the recipient |
| `amount` | u64 | yes | Amount in base units (> 0, max 1,000,000,000,000) |
| `mint` | string | yes | Solana public key of the mint |

The handler derives the recipient's associated token account automatically. If the recipient address is on the blacklist, the request is rejected.

**Response (200):**

```json
{
  "signature": "5Kj2txSigAbCdEf...",
  "slot": null
}
```

### POST /burn

Burn tokens from a source account.

**Request:**

```json
{
  "amount": 500000000,
  "mint": "4Gh2mN8qW1tR5...",
  "source": "9Qw1tR5pK3..."
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `amount` | u64 | yes | Amount in base units (> 0, max 1,000,000,000,000) |
| `mint` | string | yes | Solana public key of the mint |
| `source` | string | no | Source wallet address; defaults to the service payer |

If the source address is on the blacklist, the request is rejected.

**Response (200):**

```json
{
  "signature": "3Lm4txSig...",
  "slot": null
}
```

### GET /supply?mint=...

Get current token supply for a mint.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `mint` | string | yes | Solana public key of the mint |

**Response (200):**

```json
{
  "mint": "4Gh2mN8qW1tR5...",
  "supply": 9500000000,
  "decimals": 6
}
```

### GET /health

Health check endpoint (no authentication required).

**Response (200):**

```json
{
  "status": "ok",
  "service": "mint-burn",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "db_connected": true,
  "rpc_reachable": true
}
```

## Compliance Service (Port 3002)

### POST /blacklist

Add an address to the blacklist.

**Request:**

```json
{
  "address": "2Rf4jL6mN8...",
  "reason": "OFAC SDN List - Entity XYZ"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `address` | string | yes | Solana public key to blacklist |
| `reason` | string | no | Reason for blacklisting |

**Response (201):**

```json
{
  "id": 1,
  "address": "2Rf4jL6mN8...",
  "reason": "OFAC SDN List - Entity XYZ",
  "added_by": "4Gh2mN8qW1tR5...",
  "created_at": "2026-02-25 11:00:00"
}
```

Returns 400 if the address is already blacklisted or is not a valid Solana public key.

### DELETE /blacklist/{address}

Remove an address from the blacklist.

**Response:** `204 No Content`

Returns 404 if the address is not found in the blacklist.

### GET /blacklist

List blacklisted addresses with pagination.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `offset` | u64 | 0 | Pagination offset |
| `limit` | u64 | 100 | Max results (capped at 1000) |

**Response (200):**

```json
[
  {
    "id": 1,
    "address": "2Rf4jL6mN8...",
    "reason": "OFAC SDN List - Entity XYZ",
    "added_by": "4Gh2mN8qW1tR5...",
    "created_at": "2026-02-25 11:00:00"
  }
]
```

### GET /audit-trail

Export compliance audit trail. Supports JSON (default) and CSV output.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `format` | string | `json` (default) or `csv` |
| `action` | string | Filter by action (e.g., `mint`, `burn`, `blacklist_add`, `blacklist_remove`) |
| `actor` | string | Filter by actor public key |
| `from` | string | Start date/datetime (`YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS`) |
| `to` | string | End date/datetime (`YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS`) |
| `limit` | i64 | Max results (default 100, capped at 1000) |
| `offset` | i64 | Pagination offset |

**Response (200, JSON):**

```json
[
  {
    "id": 1,
    "action": "blacklist_add",
    "actor": "4Gh2mN8qW1tR5...",
    "target": "2Rf4jL6mN8...",
    "details": "OFAC SDN List - Entity XYZ",
    "signature": null,
    "created_at": "2026-02-25 11:00:00"
  }
]
```

**Response (200, CSV):** Returns a CSV file download with `Content-Disposition: attachment; filename="audit-trail.csv"`.

### GET /health

Health check endpoint (no authentication required).

**Response (200):**

```json
{
  "status": "ok",
  "service": "compliance",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "db_connected": true,
  "rpc_reachable": true
}
```

## Indexer Service (No HTTP Port)

The indexer subscribes to on-chain program logs via WebSocket and stores parsed events in SQLite. It does **not** expose any HTTP endpoints. When it detects a relevant event, it forwards it to the webhook service's `POST /events` endpoint internally.

## Webhook Service (Port 3004)

### POST /webhooks

Register a new webhook.

**Request:**

```json
{
  "url": "https://your-app.com/webhook/sss-events",
  "event_types": ["TokensMinted", "TokensBurned", "AddressBlacklisted"],
  "secret": "your-webhook-secret"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | string | yes | HTTPS callback URL (must not be an internal/private address) |
| `event_types` | string[] | yes | Event types to subscribe to (use `*` for all) |
| `secret` | string | no | HMAC-SHA256 secret for payload signing |

**Response (201):**

```json
{
  "id": 1,
  "url": "https://your-app.com/webhook/sss-events",
  "event_types": ["TokensMinted", "TokensBurned", "AddressBlacklisted"],
  "secret": null,
  "active": true,
  "created_at": "2026-02-25 10:00:00"
}
```

Note: The `secret` is never returned in responses for security.

### GET /webhooks

List registered webhooks with pagination.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `offset` | u64 | 0 | Pagination offset |
| `limit` | u64 | 100 | Max results (capped at 1000) |

**Response (200):**

```json
[
  {
    "id": 1,
    "url": "https://your-app.com/webhook/sss-events",
    "event_types": ["TokensMinted", "TokensBurned"],
    "secret": null,
    "active": true,
    "created_at": "2026-02-25 10:00:00"
  }
]
```

### DELETE /webhooks/{id}

Delete a webhook and all its associated delivery records.

**Response:** `204 No Content`

Returns 404 if the webhook ID is not found.

### POST /events

Internal endpoint used by the indexer to submit events for webhook delivery.

**Request:**

```json
{
  "signature": "5Kj2txSigAbCdEf...",
  "event_type": "TokensMinted",
  "data": { "recipient": "...", "amount": 1000000000 }
}
```

**Response:** `202 Accepted`

### Webhook Delivery Format

When an event matches a registered webhook, the service delivers a POST request:

```http
POST /webhook/sss-events HTTP/1.1
Content-Type: application/json
X-Webhook-Signature: sha256=abc123def456...
```

```json
{
  "event_id": 42,
  "event_type": "TokensMinted",
  "data": {
    "recipient": "9Qw1tR5pK3...",
    "amount": 1000000000
  },
  "timestamp": "2026-02-25T10:00:00+00:00"
}
```

The `X-Webhook-Signature` header is only present if a `secret` was provided during registration.

### Signature Verification

Verify webhook authenticity using HMAC-SHA256:

```typescript
import crypto from "crypto";

function verifyWebhook(payload: string, signature: string, secret: string): boolean {
  const expected = "sha256=" + crypto
    .createHmac("sha256", secret)
    .update(payload)
    .digest("hex");
  return crypto.timingSafeEqual(
    Buffer.from(signature),
    Buffer.from(expected)
  );
}
```

### Retry Policy

Failed deliveries (non-2xx response or timeout) are retried with exponential backoff of `2^attempts` seconds, up to 5 attempts:

| Attempt | Backoff Delay |
|---------|---------------|
| 1 | 2 seconds |
| 2 | 4 seconds |
| 3 | 8 seconds |
| 4 | 16 seconds |
| 5 | 32 seconds |

After 5 failed attempts the delivery is marked as `failed`. The delivery worker polls for pending deliveries on a configurable interval (`WEBHOOK_POLL_INTERVAL_SECS`, default 5s). Individual delivery requests time out after 10 seconds.

### GET /health

Health check endpoint (no authentication required).

**Response (200):**

```json
{
  "status": "ok",
  "service": "webhook",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "db_connected": true,
  "rpc_reachable": false
}
```

## Error Format

All services return errors in a consistent format:

```json
{
  "error": "Human-readable error message",
  "status": 400
}
```

The `error` field contains a client-safe message. For `Internal` and `Database` errors, the detailed message is logged server-side but not exposed to the client.

### HTTP Status Codes

| Status | Variant | Description |
|--------|---------|-------------|
| 400 | BadRequest | Invalid input, duplicate entry, blacklisted address |
| 401 | Unauthorized | Missing or invalid Bearer token |
| 403 | Forbidden | Insufficient permissions |
| 404 | NotFound | Resource not found |
| 429 | RateLimited | Too many requests |
| 500 | Internal / Database | Server-side error (details logged, not exposed) |
| 502 | Solana | Upstream Solana RPC error |

## Environment Variables

```bash
# Solana connection (used by mint-burn, compliance, indexer)
RPC_URL=https://api.devnet.solana.com
WS_URL=wss://api.devnet.solana.com

# Program ID
PROGRAM_ID=<your-program-id>

# Keypair (used by mint-burn, compliance)
KEYPAIR_PATH=~/.config/solana/id.json

# Database (all services)
DATABASE_URL=sqlite:./data/sss.db

# Authentication (all HTTP services)
API_SECRET_KEY=<your-secret-key>

# Service ports (defaults shown)
MINT_BURN_PORT=3001
COMPLIANCE_PORT=3002
WEBHOOK_PORT=3004

# Rate limiting (all HTTP services)
RATE_LIMIT_MAX=100
RATE_LIMIT_WINDOW_SECS=60

# CORS (all HTTP services)
ALLOWED_ORIGINS=http://localhost:3000

# Webhook-specific
WEBHOOK_POLL_INTERVAL_SECS=5

# Indexer-specific
WEBHOOK_SERVICE_URL=http://webhook:3000

# Logging
RUST_LOG=info
```
