# API Reference

Backend services REST API documentation for the four SSS microservices.

## Overview

The SSS backend consists of four containerized Rust/Axum services:

| Service | Port | Purpose |
|---------|------|---------|
| mint-burn | 3001 | Mint/burn lifecycle management |
| compliance | 3002 | Blacklist management, audit trail |
| indexer | 3003 | On-chain event indexing and queries |
| webhook | 3004 | Event notification delivery |

All services share:
- Structured logging via `tracing`
- SQLite database via `sqlx`
- `/health` endpoint
- `.env` configuration
- JSON request/response format

## Docker Setup

### Quick Start

```bash
cd services/

# Configure environment
cp .env.example .env
# Edit .env with your settings:
#   SOLANA_RPC_URL=https://api.devnet.solana.com
#   SSS_PROGRAM_ID=<your-program-id>
#   DATABASE_URL=sqlite:./data/sss.db
#   API_SECRET=<your-secret>

# Build and start all services
docker compose up -d

# Check health
curl http://localhost:3001/health
curl http://localhost:3002/health
curl http://localhost:3003/health
curl http://localhost:3004/health
```

### Docker Compose

```yaml
# services/docker-compose.yml
services:
  mint-burn:
    build:
      context: .
      args:
        SERVICE: mint-burn
    ports:
      - "3001:3001"
    env_file: .env
    volumes:
      - data:/app/data

  compliance:
    build:
      context: .
      args:
        SERVICE: compliance
    ports:
      - "3002:3002"
    env_file: .env
    volumes:
      - data:/app/data

  indexer:
    build:
      context: .
      args:
        SERVICE: indexer
    ports:
      - "3003:3003"
    env_file: .env
    volumes:
      - data:/app/data

  webhook:
    build:
      context: .
      args:
        SERVICE: webhook
    ports:
      - "3004:3004"
    env_file: .env
    volumes:
      - data:/app/data

volumes:
  data:
```

## Authentication

All mutating endpoints require a Bearer token:

```
Authorization: Bearer <API_SECRET>
```

Read-only endpoints (GET) do not require authentication by default.

## Mint-Burn Service (Port 3001)

### POST /mint

Request a token mint operation.

**Request:**

```json
{
  "recipient": "7Xf3kP9QwR2mN8...",
  "amount": 1000000000,
  "minter_keypair": "base58-encoded-keypair"
}
```

**Response (200):**

```json
{
  "success": true,
  "signature": "5Kj2txSigAbCdEf...",
  "amount": 1000000000,
  "recipient": "7Xf3kP9QwR2mN8...",
  "minter": "4Gh2mN8qW1tR5...",
  "remaining_quota": 999000000000,
  "timestamp": "2026-02-25T10:00:00Z"
}
```

**Errors:**

| Status | Code | Description |
|--------|------|-------------|
| 400 | `PAUSED` | Stablecoin is paused |
| 400 | `QUOTA_EXCEEDED` | Minter quota insufficient |
| 403 | `UNAUTHORIZED` | Caller is not an authorized minter |
| 500 | `TX_FAILED` | Transaction failed on-chain |

### POST /burn

Burn tokens from the caller's account.

**Request:**

```json
{
  "amount": 500000000,
  "burner_keypair": "base58-encoded-keypair"
}
```

**Response (200):**

```json
{
  "success": true,
  "signature": "3Lm4txSig...",
  "amount": 500000000,
  "burner": "9Qw1tR5pK3...",
  "timestamp": "2026-02-25T10:05:00Z"
}
```

### GET /supply

Get current token supply.

**Response (200):**

```json
{
  "mint": "7Xf3kP9QwR2mN8...",
  "total_minted": 10000000000,
  "total_burned": 500000000,
  "circulating_supply": 9500000000,
  "decimals": 6,
  "formatted": {
    "total_minted": "10000.000000",
    "total_burned": "500.000000",
    "circulating_supply": "9500.000000"
  }
}
```

### GET /minters

List all minters and their quota status.

**Response (200):**

```json
{
  "minters": [
    {
      "address": "4Gh2mN8qW1tR5...",
      "quota": 1000000000000,
      "minted": 250000000000,
      "remaining": 750000000000
    }
  ],
  "count": 1,
  "max": 10
}
```

### GET /health

Health check endpoint.

**Response (200):**

```json
{
  "service": "mint-burn",
  "status": "healthy",
  "version": "0.1.0",
  "solana_rpc": "connected",
  "database": "connected",
  "uptime_seconds": 3600
}
```

## Compliance Service (Port 3002)

### POST /blacklist

Add an address to the blacklist.

**Request:**

```json
{
  "address": "2Rf4jL6mN8...",
  "reason": "OFAC SDN List - Entity XYZ",
  "blacklister_keypair": "base58-encoded-keypair"
}
```

**Response (200):**

```json
{
  "success": true,
  "signature": "7Pq8txSig...",
  "address": "2Rf4jL6mN8...",
  "reason": "OFAC SDN List - Entity XYZ",
  "blacklisted_by": "4Gh2mN8qW1tR5...",
  "timestamp": "2026-02-25T11:00:00Z"
}
```

**Errors:**

| Status | Code | Description |
|--------|------|-------------|
| 400 | `COMPLIANCE_NOT_ENABLED` | Stablecoin is SSS-1 (no compliance) |
| 400 | `ALREADY_BLACKLISTED` | Address already on blacklist |
| 403 | `UNAUTHORIZED` | Caller is not the blacklister |

### DELETE /blacklist/:address

Remove an address from the blacklist.

**Response (200):**

```json
{
  "success": true,
  "signature": "9Wk2txSig...",
  "address": "2Rf4jL6mN8...",
  "removed_by": "4Gh2mN8qW1tR5...",
  "timestamp": "2026-02-25T12:00:00Z"
}
```

### GET /blacklist

List all blacklisted addresses.

**Response (200):**

```json
{
  "entries": [
    {
      "address": "2Rf4jL6mN8...",
      "reason": "OFAC SDN List - Entity XYZ",
      "added_at": "2026-02-25T11:00:00Z",
      "added_by": "4Gh2mN8qW1tR5..."
    }
  ],
  "count": 1
}
```

### GET /blacklist/:address

Check if a specific address is blacklisted.

**Response (200):**

```json
{
  "address": "2Rf4jL6mN8...",
  "blacklisted": true,
  "entry": {
    "reason": "OFAC SDN List - Entity XYZ",
    "added_at": "2026-02-25T11:00:00Z",
    "added_by": "4Gh2mN8qW1tR5..."
  }
}
```

### POST /seize

Seize tokens from a target address.

**Request:**

```json
{
  "target": "2Rf4jL6mN8...",
  "treasury": "9Qw1tR5pK3...",
  "seizer_keypair": "base58-encoded-keypair"
}
```

**Response (200):**

```json
{
  "success": true,
  "signature": "2Nm5txSig...",
  "from": "2Rf4jL6mN8...",
  "to": "9Qw1tR5pK3...",
  "amount": 50000000000,
  "seized_by": "8Mn3wP4...",
  "timestamp": "2026-02-25T11:05:00Z"
}
```

### GET /audit-trail

Export compliance audit trail.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `from` | ISO 8601 | Start date (inclusive) |
| `to` | ISO 8601 | End date (exclusive) |
| `action` | string | Filter: `blacklist`, `seize`, `freeze`, `thaw`, `pause` |
| `address` | string | Filter by target address |
| `format` | string | `json` (default) or `csv` |
| `limit` | number | Max results (default 100) |
| `offset` | number | Pagination offset |

**Response (200):**

```json
{
  "events": [
    {
      "id": 1,
      "event_type": "blacklist_add",
      "mint": "7Xf3kP9QwR2mN8...",
      "target_address": "2Rf4jL6mN8...",
      "reason": "OFAC SDN List - Entity XYZ",
      "performed_by": "4Gh2mN8qW1tR5...",
      "amount": null,
      "slot": 234567890,
      "block_time": 1740412800,
      "signature": "7Pq8txSig...",
      "timestamp": "2026-02-25T11:00:00Z"
    },
    {
      "id": 2,
      "event_type": "seize",
      "mint": "7Xf3kP9QwR2mN8...",
      "target_address": "2Rf4jL6mN8...",
      "reason": null,
      "performed_by": "8Mn3wP4...",
      "amount": 50000000000,
      "slot": 234567900,
      "block_time": 1740412850,
      "signature": "2Nm5txSig...",
      "timestamp": "2026-02-25T11:05:00Z"
    }
  ],
  "total": 2,
  "limit": 100,
  "offset": 0
}
```

### GET /health

```json
{
  "service": "compliance",
  "status": "healthy",
  "version": "0.1.0",
  "solana_rpc": "connected",
  "database": "connected",
  "blacklist_count": 5,
  "uptime_seconds": 3600
}
```

## Indexer Service (Port 3003)

The indexer subscribes to on-chain program logs via WebSocket and stores parsed events in SQLite.

### GET /events

Query indexed events.

**Query Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `type` | string | Event type filter (e.g., `TokensMinted`, `AddressBlacklisted`) |
| `mint` | string | Filter by mint address |
| `from_slot` | number | Start slot (inclusive) |
| `to_slot` | number | End slot (inclusive) |
| `limit` | number | Max results (default 50) |
| `offset` | number | Pagination offset |
| `order` | string | `asc` or `desc` (default `desc`) |

**Response (200):**

```json
{
  "events": [
    {
      "id": 42,
      "event_type": "TokensMinted",
      "mint": "7Xf3kP9QwR2mN8...",
      "data": {
        "recipient": "9Qw1tR5pK3...",
        "amount": 1000000000,
        "minter": "4Gh2mN8qW1tR5..."
      },
      "slot": 234567800,
      "block_time": 1740412700,
      "signature": "5Kj2txSig..."
    }
  ],
  "total": 42,
  "limit": 50,
  "offset": 0
}
```

### GET /events/stats

Get event statistics.

**Response (200):**

```json
{
  "total_events": 1234,
  "events_by_type": {
    "StablecoinInitialized": 1,
    "TokensMinted": 450,
    "TokensBurned": 100,
    "AccountFrozen": 5,
    "AccountThawed": 3,
    "AddressBlacklisted": 12,
    "AddressUnblacklisted": 2,
    "TokensSeized": 1,
    "MinterUpdated": 8,
    "RolesUpdated": 4
  },
  "last_indexed_slot": 234567900,
  "indexer_lag_slots": 2
}
```

### GET /status

Indexer sync status.

**Response (200):**

```json
{
  "service": "indexer",
  "status": "syncing",
  "current_slot": 234567900,
  "latest_chain_slot": 234567902,
  "lag_slots": 2,
  "events_indexed": 1234,
  "uptime_seconds": 3600
}
```

### GET /health

```json
{
  "service": "indexer",
  "status": "healthy",
  "version": "0.1.0",
  "solana_ws": "connected",
  "database": "connected",
  "uptime_seconds": 3600
}
```

## Webhook Service (Port 3004)

Register HTTP webhooks to receive real-time event notifications.

### POST /webhooks

Register a new webhook.

**Request:**

```json
{
  "url": "https://your-app.com/webhook/sss-events",
  "events": ["TokensMinted", "TokensBurned", "AddressBlacklisted", "TokensSeized"],
  "secret": "your-webhook-secret"
}
```

The `secret` is used to sign payloads (HMAC-SHA256) so you can verify authenticity.

**Response (201):**

```json
{
  "id": "wh_abc123",
  "url": "https://your-app.com/webhook/sss-events",
  "events": ["TokensMinted", "TokensBurned", "AddressBlacklisted", "TokensSeized"],
  "status": "active",
  "created_at": "2026-02-25T10:00:00Z"
}
```

### GET /webhooks

List registered webhooks.

**Response (200):**

```json
{
  "webhooks": [
    {
      "id": "wh_abc123",
      "url": "https://your-app.com/webhook/sss-events",
      "events": ["TokensMinted", "TokensBurned"],
      "status": "active",
      "last_delivery": "2026-02-25T12:00:00Z",
      "delivery_count": 42,
      "failure_count": 0
    }
  ],
  "count": 1
}
```

### DELETE /webhooks/:id

Unregister a webhook.

**Response (200):**

```json
{
  "success": true,
  "id": "wh_abc123",
  "deleted": true
}
```

### Webhook Delivery Format

When an event occurs, the webhook service delivers a POST request to registered URLs:

```http
POST /webhook/sss-events HTTP/1.1
Content-Type: application/json
X-SSS-Signature: sha256=abc123def456...
X-SSS-Event: TokensMinted
X-SSS-Delivery: del_xyz789

{
  "event": "TokensMinted",
  "data": {
    "mint": "7Xf3kP9QwR2mN8...",
    "recipient": "9Qw1tR5pK3...",
    "amount": 1000000000,
    "minter": "4Gh2mN8qW1tR5..."
  },
  "slot": 234567800,
  "block_time": 1740412700,
  "signature": "5Kj2txSig...",
  "timestamp": "2026-02-25T10:00:00Z"
}
```

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

Failed deliveries (non-2xx response or timeout) are retried with exponential backoff:

| Attempt | Delay |
|---------|-------|
| 1 | Immediate |
| 2 | 1 minute |
| 3 | 5 minutes |
| 4 | 30 minutes |
| 5 | 2 hours |
| 6+ | Disabled (webhook marked as failing) |

### GET /health

```json
{
  "service": "webhook",
  "status": "healthy",
  "version": "0.1.0",
  "database": "connected",
  "active_webhooks": 3,
  "pending_deliveries": 0,
  "uptime_seconds": 3600
}
```

## Error Format

All services return errors in a consistent format:

```json
{
  "error": {
    "code": "QUOTA_EXCEEDED",
    "message": "Minter quota exceeded. Remaining: 500000000",
    "details": {
      "remaining_quota": 500000000,
      "requested_amount": 1000000000
    }
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `PAUSED` | 400 | Stablecoin is paused |
| `UNAUTHORIZED` | 403 | Invalid API key or role |
| `QUOTA_EXCEEDED` | 400 | Minter quota insufficient |
| `COMPLIANCE_NOT_ENABLED` | 400 | SSS-2 feature on SSS-1 token |
| `ALREADY_BLACKLISTED` | 400 | Address already blacklisted |
| `NOT_BLACKLISTED` | 404 | Address not on blacklist |
| `TX_FAILED` | 500 | On-chain transaction failed |
| `RPC_ERROR` | 502 | Solana RPC connection error |
| `DB_ERROR` | 500 | Database operation failed |
| `INVALID_REQUEST` | 400 | Malformed request body |

## Environment Variables

```bash
# Solana connection
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WS_URL=wss://api.devnet.solana.com

# Program IDs
SSS_TOKEN_PROGRAM_ID=<program-id>
SSS_TRANSFER_HOOK_PROGRAM_ID=<program-id>

# Mint address (which stablecoin to manage)
SSS_MINT_ADDRESS=<mint-pubkey>

# Database
DATABASE_URL=sqlite:./data/sss.db

# Authentication
API_SECRET=<your-secret-key>

# Service ports
MINT_BURN_PORT=3001
COMPLIANCE_PORT=3002
INDEXER_PORT=3003
WEBHOOK_PORT=3004

# Logging
RUST_LOG=info
```
