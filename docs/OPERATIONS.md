# Operations Guide

Operator runbook for managing SSS-1 and SSS-2 stablecoins. All examples use the `sss-token` CLI unless noted otherwise.

## Prerequisites

```bash
# Configure Solana CLI
solana config set --url https://api.devnet.solana.com
solana config set --keypair ~/.config/solana/authority.json

# Verify sss-token CLI is installed
sss-token --version
```

## Initialization

### Create SSS-1 Stablecoin

```bash
sss-token init --preset sss-1 \
  --name "Superteam USD" \
  --symbol "stUSD" \
  --decimals 6
```

Output:
```
Stablecoin initialized successfully
  Mint:      7Xf3...kP9
  Config:    4Gh2...mN8
  Roles:     9Qw1...tR5
  Preset:    SSS-1
  Authority: <your wallet>
```

### Create SSS-2 Stablecoin

```bash
sss-token init --preset sss-2 \
  --name "Regulated USD" \
  --symbol "rUSD" \
  --decimals 6
```

### Custom Configuration

```bash
# Create config file
cat > custom-config.toml <<EOF
name = "Custom Token"
symbol = "CUST"
decimals = 9
enable_permanent_delegate = true
enable_transfer_hook = false
default_account_frozen = false
EOF

sss-token init --custom custom-config.toml
```

### Verify Initialization

```bash
sss-token status
```

Output:
```
Stablecoin Status
═════════════════════════════════════
  Name:            Superteam USD
  Symbol:          stUSD
  Mint:            7Xf3...kP9
  Decimals:        6
  Paused:          false
  Total Minted:    0
  Total Burned:    0
  Supply:          0
  Preset:          SSS-1
  Compliance:      disabled
  Authority:       <pubkey>
```

## Mint Operations

### Add a Minter

Before minting, add at least one minter with a quota.

```bash
# Add minter with 1,000,000 token quota
sss-token minters add <MINTER_PUBKEY> --quota 1000000000000
```

### List Minters

```bash
sss-token minters list
```

Output:
```
Minters (2/10)
═══════════════════════════════════════════════════════
  Address                                  Quota         Used          Remaining
  4Gh2...mN8                              1,000,000     250,000       750,000
  9Qw1...tR5                              500,000       0             500,000
```

### Mint Tokens

```bash
# Mint 10,000 tokens to recipient
sss-token mint <RECIPIENT_PUBKEY> 10000000000
```

The CLI will:
1. Verify the caller is an authorized minter
2. Check remaining quota is sufficient
3. Create recipient token account if needed
4. Execute mint via CPI
5. Display transaction signature

### Update Minter Quota

```bash
# Increase quota
sss-token minters add <MINTER_PUBKEY> --quota 2000000000000
```

### Remove a Minter

```bash
sss-token minters remove <MINTER_PUBKEY>
```

## Burn Operations

### Add a Burner

```bash
# Burner role is set via update_roles or during initialization
# The authority can configure burner addresses
sss-token minters add <BURNER_PUBKEY> --quota 0
# Or through role management (see Role Management section)
```

### Burn Tokens

```bash
# Burn 5,000 tokens from caller's account
sss-token burn 5000000000
```

### Check Supply

```bash
sss-token supply
```

Output:
```
Token Supply
═══════════════════════
  Total Minted:  1,000,000.000000
  Total Burned:  50,000.000000
  Circulating:   950,000.000000
```

## Freeze / Thaw Operations

### Freeze an Account

Prevents all transfers in and out of a specific token account.

```bash
sss-token freeze <ACCOUNT_OWNER_PUBKEY>
```

### Thaw an Account

```bash
sss-token thaw <ACCOUNT_OWNER_PUBKEY>
```

### Verify Freeze Status

```bash
sss-token holders --min-balance 0
```

The output includes freeze status for each account.

## Emergency Pause / Unpause

### Pause All Operations

Global circuit breaker. Blocks: mint, burn, and all state-changing instructions (except unpause).

```bash
sss-token pause
```

Output:
```
PAUSED: All operations are now suspended
Transaction: 3Kf9...signature
```

### Unpause

```bash
sss-token unpause
```

### Monitoring Pause Status

```bash
sss-token status | grep Paused
```

## Role Management

### View Current Roles

```bash
sss-token status
```

The status output includes all role assignments.

### Update Pauser

```bash
# Requires master_authority
sss-token update-roles --pauser <NEW_PAUSER_PUBKEY>
```

### Transfer Authority

Two-step process to prevent accidental lockout.

```bash
# Step 1: Current authority initiates transfer
sss-token transfer-authority <NEW_AUTHORITY_PUBKEY>

# Step 2: New authority accepts (must be run by new authority)
sss-token accept-authority
```

## SSS-2 Compliance Operations

These operations require an SSS-2 stablecoin with compliance features enabled.

### Blacklist Management

#### Add to Blacklist

```bash
sss-token blacklist add <ADDRESS> --reason "OFAC SDN List - Entity XYZ"
```

This creates a BlacklistEntry PDA on-chain. Once blacklisted:
- The address cannot send tokens (transfer hook rejects)
- The address cannot receive tokens (transfer hook rejects)
- The blacklist entry is publicly visible on-chain

#### Remove from Blacklist

```bash
sss-token blacklist remove <ADDRESS>
```

Closes the BlacklistEntry PDA and reclaims rent.

#### View Blacklist

```bash
sss-token audit-log --action blacklist
```

### Seize Assets

Forcibly transfer tokens from a target account to a treasury using the permanent delegate.

```bash
sss-token seize <TARGET_ADDRESS> --to <TREASURY_PUBKEY>
```

This operation:
1. Validates the caller is the seizer role
2. Uses the permanent delegate (StablecoinConfig PDA) to transfer all tokens
3. Does NOT require the target's signature
4. Emits a `TokensSeized` event for audit trail
5. Is irreversible

### Audit Trail

#### Export All Events

```bash
sss-token audit-log
```

Output:
```
Audit Trail
═══════════════════════════════════════════════════════════════════
  Time                 Action              Details
  2026-02-25 10:00:00  Initialized         Preset: SSS-2, Authority: 4Gh2...
  2026-02-25 10:05:00  MinterAdded         Address: 9Qw1..., Quota: 1,000,000
  2026-02-25 10:10:00  Minted              To: 7Xf3..., Amount: 100,000
  2026-02-25 11:00:00  Blacklisted         Address: 2Rf4..., Reason: OFAC
  2026-02-25 11:05:00  Seized              From: 2Rf4..., To: Treasury, Amount: 50,000
```

#### Filter by Action Type

```bash
sss-token audit-log --action blacklist
sss-token audit-log --action mint
sss-token audit-log --action seize
```

## Monitoring and Health Checks

### Check Stablecoin Status

```bash
# Full status
sss-token status

# Supply only
sss-token supply
```

### List All Token Holders

```bash
# All holders
sss-token holders

# Holders with minimum balance
sss-token holders --min-balance 1000
```

Output:
```
Token Holders (42 accounts)
═══════════════════════════════════════════════════════
  Address                                  Balance          Frozen
  7Xf3...kP9                              500,000.000000   false
  4Gh2...mN8                              250,000.000000   false
  2Rf4...jL6                              100,000.000000   true
  ...
```

### Backend Service Health

If running the backend services:

```bash
# Check all services
curl http://localhost:3001/health  # mint-burn
curl http://localhost:3002/health  # compliance
curl http://localhost:3003/health  # indexer
curl http://localhost:3004/health  # webhook
```

## Operational Procedures

### Daily Operations Checklist

1. Check stablecoin status (`sss-token status`)
2. Review minter quota utilization (`sss-token minters list`)
3. Verify supply consistency (`sss-token supply`)
4. Review audit log for unexpected events (`sss-token audit-log`)
5. Check blacklist status if SSS-2 (`sss-token audit-log --action blacklist`)

### Emergency Response: Suspected Compromise

1. **Immediate**: Pause the stablecoin
   ```bash
   sss-token pause
   ```

2. **Investigate**: Review audit trail
   ```bash
   sss-token audit-log
   ```

3. **Contain**: Freeze compromised accounts
   ```bash
   sss-token freeze <COMPROMISED_ACCOUNT>
   ```

4. **If SSS-2**: Blacklist compromised addresses
   ```bash
   sss-token blacklist add <ADDRESS> --reason "Compromised key"
   ```

5. **Recover**: Seize stolen tokens if SSS-2
   ```bash
   sss-token seize <ADDRESS> --to <TREASURY>
   ```

6. **Remediate**: Remove compromised minter keys
   ```bash
   sss-token minters remove <COMPROMISED_MINTER>
   ```

7. **Resume**: Unpause after investigation
   ```bash
   sss-token unpause
   ```

### Key Rotation Procedure

1. Generate new keypair for the role
2. Update the role assignment:
   ```bash
   sss-token update-roles --pauser <NEW_KEY>
   ```
3. Verify the change took effect:
   ```bash
   sss-token status
   ```
4. Securely destroy the old keypair

### Authority Transfer Procedure

1. Verify the new authority address
2. Initiate transfer:
   ```bash
   sss-token transfer-authority <NEW_AUTHORITY>
   ```
3. New authority accepts:
   ```bash
   # With new authority's keypair
   sss-token accept-authority
   ```
4. Verify:
   ```bash
   sss-token status
   ```

## Backend Services Configuration

The backend services (mint-burn, compliance, indexer, webhook) are four Axum microservices configured via environment variables. All services share the same variable set; not every service uses every variable.

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `API_SECRET_KEY` | Yes | None | Bearer token for API authentication (constant-time comparison) |
| `DATABASE_URL` | No | `sqlite:./data/sss.db` | SQLite database path |
| `RPC_URL` | No | `https://api.devnet.solana.com` | Solana RPC endpoint |
| `WS_URL` | No | `wss://api.devnet.solana.com` | Solana WebSocket endpoint (indexer only) |
| `KEYPAIR_PATH` | No | `~/.config/solana/id.json` | Payer keypair file path |
| `PROGRAM_ID` | No | `11111111111111111111111111111111` | SSS Token program ID |
| `RATE_LIMIT_MAX` | No | `100` | Maximum requests per window per IP |
| `RATE_LIMIT_WINDOW_SECS` | No | `60` | Rate limit window in seconds |
| `ALLOWED_ORIGINS` | No | `*` | CORS allowed origins (comma-separated) |
| `WEBHOOK_POLL_INTERVAL_SECS` | No | `5` | Webhook delivery poll interval (webhook service only) |
| `WEBHOOK_SERVICE_URL` | No | `http://webhook:3000` | URL for indexer to forward events (indexer only) |
| `RUST_LOG` | No | `info` | Log level filter (e.g. `debug`, `info,tower_http=warn`) |

### Docker Deployment

The services are orchestrated with Docker Compose. All four services share a single SQLite volume.

```bash
cd services

# Create .env from the example template
cp .env.example .env
# Edit .env and set API_SECRET_KEY (required)

# Start all services
docker compose up -d

# View logs
docker compose logs -f

# Check health
curl http://localhost:3001/health   # mint-burn
curl http://localhost:3002/health   # compliance
curl http://localhost:3004/health   # webhook

# Stop all services
docker compose down
```

Service port mapping:

| Service | Container Port | Host Port |
|---------|---------------|-----------|
| mint-burn | 3000 | 3001 |
| compliance | 3000 | 3002 |
| indexer | (no exposed port) | -- |
| webhook | 3000 | 3004 |
