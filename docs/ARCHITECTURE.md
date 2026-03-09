# Architecture

## Overview

The Solana Stablecoin Standard follows a three-layer architecture inspired by the OpenZeppelin model: the library is the SDK, the standards are configurable presets.

```
┌───────────────────────────────────────────────────────────────┐
│  Layer 3: Applications                                        │
│  CLI, TUI, Backend Services, Frontend                         │
└──────────────────────────┬────────────────────────────────────┘
                           │
┌──────────────────────────▼────────────────────────────────────┐
│  Layer 2: SDK + Modules                                       │
│  SolanaStablecoin class, ComplianceModule, PDA helpers        │
│  Presets: SSS-1 (minimal) and SSS-2 (compliant)              │
└──────────────────────────┬────────────────────────────────────┘
                           │
┌──────────────────────────▼────────────────────────────────────┐
│  Layer 1: On-Chain Programs                                   │
│  sss-token (Anchor) + sss-transfer-hook (Anchor)             │
│  Built on Solana Token-2022 extensions                        │
└───────────────────────────────────────────────────────────────┘
```

### Layer 1: On-Chain Programs

Two Anchor programs deployed to Solana:

1. **sss-token** — Single configurable program supporting both SSS-1 and SSS-2 presets via initialization parameters. Feature gating at runtime means SSS-2 instructions fail gracefully with `ComplianceNotEnabled` when the compliance module is not enabled.

2. **sss-transfer-hook** — Separate program required by Token-2022 architecture. Enforces blacklist checks on every token transfer for SSS-2 stablecoins.

### Layer 2: SDK + Modules

The TypeScript SDK (`@stbr/sss-token`) provides:

- **SolanaStablecoin** — Factory class with `create()` and `load()` methods. All operations (mint, burn, freeze, thaw, pause) are methods on this class.
- **ComplianceModule** — SSS-2 operations (blacklist, seize). Accessed via `stablecoin.compliance`. Returns a real module for SSS-2 tokens; throws `ComplianceNotEnabled` for SSS-1.
- **Presets** — Enum mapping to preset configurations (extensions, default roles).
- **PDA helpers** — Derivation functions for all program-derived addresses.

### Layer 3: Applications

- **Rust CLI** (`sss-token`) — Command-line tool for all stablecoin operations.
- **Backend Services** — Four Axum microservices: mint-burn, compliance, indexer, webhook.
- **Admin TUI** — ratatui-based terminal dashboard.
- **Frontend** — Next.js example application using the SDK.

## Single Configurable Program Design

Instead of separate programs per preset, SSS uses a single `sss-token` program with runtime feature gating:

```
┌─────────────────────────────────────────────────────┐
│                   sss-token Program                  │
│                                                      │
│  ┌─────────────────┐     ┌────────────────────────┐ │
│  │  Core Module     │     │  Compliance Module     │ │
│  │                  │     │  (SSS-2 only)          │ │
│  │  initialize      │     │                        │ │
│  │  mint            │     │  add_to_blacklist      │ │
│  │  burn            │     │  remove_from_blacklist │ │
│  │  freeze_account  │     │  seize                 │ │
│  │  thaw_account    │     │                        │ │
│  │  pause / unpause │     │  Feature gate:         │ │
│  │  update_minter   │     │  config.is_compliance  │ │
│  │  update_roles    │     │  _enabled()            │ │
│  │  transfer_auth   │     │                        │ │
│  └─────────────────┘     └────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

Feature gating pattern:

```rust
pub fn add_to_blacklist(ctx: Context<AddToBlacklist>, ...) -> Result<()> {
    require!(
        ctx.accounts.stablecoin_config.enable_transfer_hook,
        StablecoinError::ComplianceNotEnabled
    );
    // ... proceed with blacklist logic
}
```

This design matches the PYUSD production pattern and was chosen because the requirements specify "a single configurable program that supports both presets via initialization parameters."

## Account Structure

### StablecoinConfig

PDA seeds: `["stablecoin", mint.key()]`

```
┌──────────────────────────────────────────────┐
│              StablecoinConfig                  │
├──────────────────┬───────────────────────────┤
│ authority        │ Pubkey (32 bytes)          │
│ mint             │ Pubkey (32 bytes)          │
│ name             │ String (4 + 32 bytes max)  │
│ symbol           │ String (4 + 10 bytes max)  │
│ uri              │ String (4 + 200 bytes max) │
│ decimals         │ u8 (1 byte)                │
│ paused           │ bool (1 byte)              │
│ total_minted     │ u64 (8 bytes)              │
│ total_burned     │ u64 (8 bytes)              │
│ enable_perm_del  │ bool (1 byte)  [SSS-2]     │
│ enable_tx_hook   │ bool (1 byte)  [SSS-2]     │
│ default_frozen   │ bool (1 byte)  [SSS-2]     │
│ hook_program     │ Option<Pubkey> (33 bytes)  │
│ bump             │ u8 (1 byte)                │
│ _reserved        │ [u8; 64]                   │
├──────────────────┴───────────────────────────┤
│ Total: 8 (disc) + ~460 bytes                  │
└──────────────────────────────────────────────┘
```

### RoleConfig

PDA seeds: `["roles", stablecoin_config.key()]`

```
┌──────────────────────────────────────────────┐
│                RoleConfig                      │
├──────────────────┬───────────────────────────┤
│ stablecoin       │ Pubkey (32 bytes)          │
│ master_authority │ Pubkey (32 bytes)          │
│ pauser           │ Pubkey (32 bytes)          │
│ minters          │ Vec<MinterInfo> (max 10)   │
│                  │   address: Pubkey           │
│                  │   quota: u64                │
│                  │   minted: u64               │
│ burners          │ Vec<Pubkey> (max 10)       │
│ blacklister      │ Pubkey (32 bytes) [SSS-2]  │
│ seizer           │ Pubkey (32 bytes) [SSS-2]  │
│ bump             │ u8 (1 byte)                │
│ _reserved        │ [u8; 64]                   │
└──────────────────┴───────────────────────────┘
```

Role assignments:

| Role | Permission | Preset |
|------|-----------|--------|
| `master_authority` | All operations, role management | ALL |
| `pauser` | pause, unpause, freeze, thaw | ALL |
| `minters[]` | mint (up to individual quota) | ALL |
| `burners[]` | burn tokens | ALL |
| `blacklister` | add/remove from blacklist | SSS-2 |
| `seizer` | seize tokens via permanent delegate | SSS-2 |

### BlacklistEntry

PDA seeds: `["blacklist", mint.key(), address.key()]`

```
┌──────────────────────────────────────────────┐
│              BlacklistEntry                    │
├──────────────────┬───────────────────────────┤
│ stablecoin       │ Pubkey (32 bytes)          │
│ address          │ Pubkey (32 bytes)          │
│ reason           │ String (4 + 128 bytes max) │
│ added_at         │ i64 (8 bytes)              │
│ added_by         │ Pubkey (32 bytes)          │
│ bump             │ u8 (1 byte)                │
└──────────────────┴───────────────────────────┘
```

### MinterInfo (embedded in RoleConfig)

```
┌──────────────────────────────────────┐
│            MinterInfo                 │
├──────────────┬───────────────────────┤
│ address      │ Pubkey (32 bytes)      │
│ quota        │ u64 (8 bytes)          │
│ minted       │ u64 (8 bytes)          │
├──────────────┴───────────────────────┤
│ remaining_quota = quota - minted      │
└──────────────────────────────────────┘
```

## Token-2022 Extensions

### SSS-1 Extensions

```
┌─────────────────────────────────────────┐
│           SSS-1 Mint Account             │
│                                          │
│  ┌─────────────────────────────────┐    │
│  │  MintAuthority = StablecoinPDA  │    │
│  ├─────────────────────────────────┤    │
│  │  FreezeAuthority = StablecoinPDA│    │
│  ├─────────────────────────────────┤    │
│  │  MetadataPointer → self         │    │
│  ├─────────────────────────────────┤    │
│  │  TokenMetadata                  │    │
│  │    name, symbol, uri            │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### SSS-2 Extensions (all SSS-1 + compliance)

```
┌─────────────────────────────────────────┐
│           SSS-2 Mint Account             │
│                                          │
│  ┌─────────────────────────────────┐    │
│  │  MintAuthority = StablecoinPDA  │    │
│  ├─────────────────────────────────┤    │
│  │  FreezeAuthority = StablecoinPDA│    │
│  ├─────────────────────────────────┤    │
│  │  MetadataPointer → self         │    │
│  ├─────────────────────────────────┤    │
│  │  TokenMetadata                  │    │
│  ├─────────────────────────────────┤    │
│  │  PermanentDelegate              │    │
│  │    delegate = StablecoinPDA     │    │
│  ├─────────────────────────────────┤    │
│  │  TransferHook                   │    │
│  │    program = sss-transfer-hook  │    │
│  ├─────────────────────────────────┤    │
│  │  DefaultAccountState            │    │
│  │    state = Frozen (configurable)│    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

**Extension Compatibility Matrix:**

| Extension A | Extension B | Compatible? |
|-------------|-------------|-------------|
| TransferHook | PermanentDelegate | Yes |
| TransferHook | ConfidentialTransfers | **No** (SSS-2 and SSS-3 are mutually exclusive) |
| DefaultAccountState | FreezeAuthority | Yes (never revoke freeze if default=frozen) |
| MetadataPointer | All others | Yes |

## Data Flow Diagrams

### Mint Flow

```
                  ┌─────────┐
                  │  Minter  │
                  └────┬─────┘
                       │ 1. Call mint(recipient, amount)
                       ▼
              ┌────────────────┐
              │   sss-token    │
              │   Program      │
              └───────┬────────┘
                      │ 2. Validate:
                      │    - !paused
                      │    - minter in role_config.minters
                      │    - remaining_quota >= amount
                      ▼
              ┌────────────────┐
              │ Token-2022 CPI │
              │ mint_to        │
              │ (PDA signer)   │
              └───────┬────────┘
                      │ 3. Update state:
                      │    - minter.minted += amount
                      │    - config.total_minted += amount
                      ▼
              ┌────────────────┐
              │ Emit event:    │
              │ TokensMinted   │
              └────────────────┘
```

### SSS-2 Compliance Flow (Blacklist + Transfer Enforcement)

```
 1. Blacklist                              2. Transfer Attempt
 ┌──────────┐                              ┌──────┐    ┌──────┐
 │Blacklister│                              │User A│    │User B│
 └─────┬─────┘                              └──┬───┘    └──┬───┘
       │ add_to_blacklist(addr, reason)         │           │
       ▼                                        │ transfer  │
 ┌───────────┐                                  ▼           │
 │ sss-token │                            ┌──────────┐      │
 │ Program   │                            │Token-2022│      │
 └─────┬─────┘                            │transfer  │      │
       │                                  │_checked  │      │
       ▼                                  └────┬─────┘      │
 ┌──────────────┐                              │            │
 │Create PDA:   │                              ▼            │
 │BlacklistEntry│                     ┌──────────────────┐  │
 │["blacklist", │                     │sss-transfer-hook │  │
 │ mint, addr]  │                     │Program           │  │
 └──────────────┘                     └────────┬─────────┘  │
                                               │            │
                                               ▼            │
                                    ┌────────────────────┐  │
                                    │Check BlacklistEntry│  │
                                    │PDAs for sender and │  │
                                    │receiver            │  │
                                    └────────┬───────────┘  │
                                             │              │
                                    ┌────────▼───────────┐  │
                                    │ PDA exists?        │  │
                                    │ Yes → REJECT       │  │
                                    │ No  → ALLOW        │  │
                                    └────────────────────┘  │
```

### Seize Flow (SSS-2)

```
  ┌────────┐
  │ Seizer │
  └───┬────┘
      │ 1. seize(from, to)
      ▼
┌───────────┐
│ sss-token │ 2. Validate:
│ Program   │    - compliance enabled
│           │    - caller is seizer
└─────┬─────┘
      │ 3. CPI: token_2022::transfer_checked
      │    authority = StablecoinConfig PDA
      │    (permanent delegate)
      ▼
┌───────────────┐
│ Token-2022    │ 4. Transfers tokens from
│ Permanent     │    target to treasury
│ Delegate      │    WITHOUT target's signature
└───────┬───────┘
        │ 5. Emit TokensSeized event
        ▼
┌───────────────┐
│ Audit trail   │
│ (on-chain log)│
└───────────────┘
```

## Transfer Hook Architecture

The transfer hook program is invoked automatically by Token-2022 on every `transfer_checked` call for SSS-2 tokens.

```
┌─────────────────────────────────────────────────────────────┐
│                    Token-2022 Transfer Flow                   │
│                                                              │
│  1. User calls transfer_checked                              │
│     ▼                                                        │
│  2. Token-2022 reads TransferHook extension on mint          │
│     → finds sss-transfer-hook program ID                     │
│     ▼                                                        │
│  3. Token-2022 reads ExtraAccountMetaList PDA                │
│     → resolves sender & receiver BlacklistEntry PDAs         │
│     ▼                                                        │
│  4. Token-2022 CPIs into sss-transfer-hook::transfer_hook    │
│     Passes: source, mint, dest, owner, amount,               │
│             sender_blacklist_pda, receiver_blacklist_pda      │
│     ▼                                                        │
│  5. Hook checks if either PDA has data (account exists)      │
│     → If yes: return BlacklistedSender/BlacklistedReceiver   │
│     → If no:  return Ok (transfer proceeds)                  │
└─────────────────────────────────────────────────────────────┘
```

### ExtraAccountMeta Resolution

The ExtraAccountMetaList PDA is initialized once after stablecoin creation. It tells Token-2022 how to derive the additional accounts needed by the hook:

```rust
// Sender BlacklistEntry PDA
ExtraAccountMeta::new_with_seeds(
    &[
        Seed::Literal { bytes: b"blacklist".to_vec() },
        Seed::AccountKey { index: 1 },       // mint
        Seed::AccountData {
            account_index: 0,                  // source token account
            data_index: 32,                    // owner field offset
            length: 32,                        // Pubkey length
        },
    ],
    &sss_token_program_id,
    false, // is_signer
    false, // is_writable
)?;
```

## Security Model

### Role-Based Access Control

```
┌─────────────────────────────────────────────┐
│              Authority Hierarchy             │
│                                              │
│  master_authority (1)                        │
│  ├── Can assign all roles                   │
│  ├── Can update_minter, update_roles        │
│  ├── Can transfer_authority (two-step)      │
│  │                                           │
│  ├── pauser (1)                             │
│  │   ├── pause / unpause                    │
│  │   └── freeze_account / thaw_account      │
│  │                                           │
│  ├── minters[] (max 10)                     │
│  │   └── mint (up to individual quota)      │
│  │                                           │
│  ├── burners[] (max 10)                     │
│  │   └── burn                               │
│  │                                           │
│  ├── blacklister (1) [SSS-2]               │
│  │   ├── add_to_blacklist                   │
│  │   └── remove_from_blacklist              │
│  │                                           │
│  └── seizer (1) [SSS-2]                    │
│      └── seize (via permanent delegate)     │
└─────────────────────────────────────────────┘
```

### Key Security Properties

1. **PDA-controlled mint** — The mint authority is the StablecoinConfig PDA, not any user key. Only the program can sign mint operations.

2. **Two-step authority transfer** — Prevents accidental lockout. Authority sets pending, new authority accepts.

3. **Per-minter quotas** — Each minter has an independent ceiling. Compromise of one minter key limits exposure.

4. **Global pause** — Emergency circuit breaker that blocks all operational instructions.

5. **Permanent delegate isolation** — The permanent delegate (for seize) is the StablecoinConfig PDA. Only the seizer role can trigger it through the program, and every invocation emits an auditable event.

6. **Transfer hook immutability** — Once initialized, the transfer hook program cannot be changed (Token-2022 constraint). Blacklist enforcement cannot be bypassed.

### Recommended Key Management

| Role | Recommendation |
|------|---------------|
| master_authority | Multisig (e.g., Squads) with 3-of-5 threshold |
| pauser | Hot wallet for rapid emergency response |
| minters | Individual hot wallets with tight quotas |
| burners | Individual hot wallets |
| blacklister | Warm wallet with automated sanctions feed |
| seizer | Cold wallet or multisig (high-impact action) |

## Backend Services Architecture

Four Axum microservices provide REST API access to on-chain operations, event indexing, and webhook delivery.

```
┌──────────────────────────────────────────────────────────────┐
│                      Client Request                           │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│  Middleware Stack (applied in order)                          │
│                                                              │
│  1. TraceLayer          — HTTP-level tracing spans           │
│  2. CORS                — Origin validation (ALLOWED_ORIGINS)│
│  3. Security Headers    — HSTS, X-Content-Type-Options,      │
│                           X-Frame-Options, Cache-Control,    │
│                           Referrer-Policy                    │
│  4. Observability       — UUID request ID (X-Request-Id),    │
│                           request count / error count /      │
│                           duration metrics                   │
│  5. Rate Limiting       — Token-bucket per client IP         │
│                           (RATE_LIMIT_MAX / WINDOW_SECS)     │
│  6. Authentication      — Bearer token (constant-time check),│
│                           skips GET /health and GET /metrics │
│  7. Handler             — Route-specific business logic      │
└──────────────────────────────────────────────────────────────┘
```

### Inter-Service Communication

The indexer watches on-chain events via Solana WebSocket (`WS_URL`) and forwards them to the webhook service over an authenticated HTTP POST. The webhook service is an internal dependency of the indexer (Docker Compose `depends_on` with health check).

```
┌──────────┐   WebSocket    ┌──────────┐   HTTP POST    ┌──────────┐
│  Solana  │ ─────────────▶ │ Indexer  │ ─────────────▶ │ Webhook  │
│  RPC     │   (events)     │          │  (authenticated)│ Service  │
└──────────┘                └──────────┘                 └─────┬────┘
                                                               │
                                                               ▼
                                                      ┌──────────────┐
                                                      │ Registered   │
                                                      │ Endpoints    │
                                                      └──────────────┘
```

### Database

All services share a single SQLite database (`DATABASE_URL`) with WAL mode and `busy_timeout` configured for concurrent reads. The shared volume (`sss-data`) is mounted into every container.

### Webhook Delivery

The webhook service runs a background worker that polls for pending deliveries on a configurable interval (`WEBHOOK_POLL_INTERVAL_SECS`, default 5s). Failed deliveries are retried with exponential backoff.
