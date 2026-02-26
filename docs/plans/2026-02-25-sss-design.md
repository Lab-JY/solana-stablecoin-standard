# Solana Stablecoin Standard (SSS) — Design Document

**Date**: 2026-02-25
**Status**: Approved

## Overview

A modular stablecoin SDK for Solana with two opinionated presets (SSS-1, SSS-2) built on Token-2022 extensions. The SDK follows the OpenZeppelin model: the library is the SDK, the standards are configurable presets.

## Architecture Decision: Single Configurable Program

Two Anchor programs total:

1. **sss-token**: Main program supporting both SSS-1 and SSS-2 via initialization parameters. Feature gating at runtime — SSS-2 instructions fail gracefully when compliance module is not enabled.
2. **sss-transfer-hook**: Separate program (required by Token-2022 architecture) that enforces blacklist checks on every transfer.

Rationale: Requirements explicitly specify "a single configurable program that supports both presets via initialization parameters." PYUSD uses the same single-program pattern in production.

## Tech Stack

| Component | Language | Framework |
|-----------|----------|-----------|
| On-chain programs | Rust | Anchor 0.31.x |
| TypeScript SDK | TypeScript | @coral-xyz/anchor, @solana/web3.js |
| CLI (sss-token) | Rust | clap v4 |
| Backend services | Rust | Axum + Tokio |
| Admin TUI | Rust | ratatui |
| Frontend example | TypeScript | Next.js |
| Database | SQLite | sqlx |

## Project Structure

```
solana-stablecoin-standard/
├── programs/
│   ├── sss-token/                  # Main Anchor program (SSS-1 + SSS-2)
│   │   └── src/
│   │       ├── lib.rs              # Program entry, instruction dispatch
│   │       ├── state.rs            # StablecoinConfig, RoleConfig, BlacklistEntry
│   │       ├── error.rs            # Error codes
│   │       ├── events.rs           # Event structs
│   │       ├── constants.rs        # Seeds, limits
│   │       └── instructions/
│   │           ├── mod.rs
│   │           ├── initialize.rs
│   │           ├── mint.rs
│   │           ├── burn.rs
│   │           ├── freeze_account.rs
│   │           ├── thaw_account.rs
│   │           ├── pause.rs
│   │           ├── unpause.rs
│   │           ├── update_minter.rs
│   │           ├── update_roles.rs
│   │           ├── transfer_authority.rs
│   │           ├── add_to_blacklist.rs
│   │           ├── remove_from_blacklist.rs
│   │           └── seize.rs
│   └── sss-transfer-hook/          # Transfer Hook program
│       └── src/
│           ├── lib.rs
│           └── instructions/
│               ├── initialize.rs
│               └── transfer_hook.rs
├── sdk/
│   └── core/                       # @stbr/sss-token TypeScript SDK
│       ├── src/
│       │   ├── index.ts
│       │   ├── stablecoin.ts       # SolanaStablecoin class
│       │   ├── presets.ts          # SSS-1, SSS-2 preset configs
│       │   ├── compliance.ts       # ComplianceModule
│       │   ├── pda.ts              # PDA derivation
│       │   ├── types.ts
│       │   └── errors.ts
│       ├── tests/
│       └── package.json
├── cli/                            # Rust CLI
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── commands/
│       ├── config.rs
│       └── output.rs
├── services/                       # Rust backend services
│   ├── shared/                     # Shared library
│   ├── mint-burn/                  # Mint/Burn lifecycle service
│   ├── compliance/                 # Compliance service (SSS-2)
│   ├── indexer/                    # Event indexer
│   ├── webhook/                    # Webhook notification service
│   └── docker-compose.yml
├── tui/                            # Admin TUI (bonus)
│   ├── Cargo.toml
│   └── src/
├── tests/                          # Integration tests
│   ├── sss-1.ts
│   ├── sss-2.ts
│   ├── lifecycle.ts
│   ├── compliance.ts
│   └── helpers/
├── trident-tests/                  # Fuzz tests
├── scripts/                        # Devnet deployment scripts
├── app/                            # Next.js frontend (bonus)
├── docs/
│   ├── README.md
│   ├── ARCHITECTURE.md
│   ├── SDK.md
│   ├── OPERATIONS.md
│   ├── SSS-1.md
│   ├── SSS-2.md
│   ├── SSS-3.md                    # Private stablecoin PoC spec
│   ├── COMPLIANCE.md
│   └── API.md
├── Anchor.toml
├── Cargo.toml                      # Workspace: programs, cli, services, tui
├── package.json                    # Node workspace: sdk, tests
└── CLAUDE.md
```

## On-Chain Program Design

### State Accounts

```rust
// PDA seeds: ["stablecoin", mint]
#[account]
pub struct StablecoinConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub name: String,               // max 32 bytes
    pub symbol: String,             // max 10 bytes
    pub uri: String,                // max 200 bytes
    pub decimals: u8,
    pub paused: bool,
    pub total_minted: u64,
    pub total_burned: u64,
    // SSS-2 feature flags
    pub enable_permanent_delegate: bool,
    pub enable_transfer_hook: bool,
    pub default_account_frozen: bool,
    pub transfer_hook_program: Option<Pubkey>,
    pub bump: u8,
    pub _reserved: [u8; 64],
}

// PDA seeds: ["roles", stablecoin]
#[account]
pub struct RoleConfig {
    pub stablecoin: Pubkey,
    pub master_authority: Pubkey,
    pub pauser: Pubkey,
    pub minters: Vec<MinterInfo>,   // max 10
    pub burners: Vec<Pubkey>,       // max 10
    pub blacklister: Pubkey,        // SSS-2 only
    pub seizer: Pubkey,             // SSS-2 only
    pub bump: u8,
    pub _reserved: [u8; 64],
}

pub struct MinterInfo {
    pub address: Pubkey,
    pub quota: u64,
    pub minted: u64,
}

// PDA seeds: ["blacklist", mint, address]
#[account]
pub struct BlacklistEntry {
    pub stablecoin: Pubkey,
    pub address: Pubkey,
    pub reason: String,             // max 128 bytes
    pub added_at: i64,
    pub added_by: Pubkey,
    pub bump: u8,
}
```

### Token-2022 Extensions per Preset

**SSS-1 (Minimal)**:
- MintAuthority
- FreezeAuthority
- MetadataPointer (self-referencing)
- TokenMetadata (name, symbol, uri)

**SSS-2 (Compliant)**:
- All SSS-1 extensions, plus:
- PermanentDelegate (seize/burn from any account)
- TransferHook (blacklist enforcement via sss-transfer-hook program)
- DefaultAccountState = Frozen (optional, configurable)

### Instruction Set

| Instruction | Preset | Authority | Description |
|-------------|--------|-----------|-------------|
| initialize | ALL | authority | Create mint + configure extensions + init state |
| mint | ALL | minter | Mint tokens (checks per-minter quota) |
| burn | ALL | burner | Burn tokens |
| freeze_account | ALL | authority/pauser | Freeze a token account |
| thaw_account | ALL | authority/pauser | Thaw a frozen account |
| pause | ALL | pauser | Global pause all operations |
| unpause | ALL | pauser | Resume operations |
| update_minter | ALL | authority | Add/remove/update minter quota |
| update_roles | ALL | authority | Update role assignments |
| transfer_authority | ALL | authority | Transfer master authority |
| add_to_blacklist | SSS-2 | blacklister | Create BlacklistEntry PDA |
| remove_from_blacklist | SSS-2 | blacklister | Close BlacklistEntry PDA |
| seize | SSS-2 | seizer | Transfer tokens via permanent delegate |

### Feature Gating Pattern

SSS-2 instructions check feature flags and fail gracefully:

```rust
pub fn add_to_blacklist(ctx: Context<AddToBlacklist>, ...) -> Result<()> {
    require!(
        ctx.accounts.stablecoin_config.enable_transfer_hook,
        StablecoinError::ComplianceNotEnabled
    );
    // ...
}
```

### Transfer Hook Program

- `initialize_extra_account_meta_list`: Registers sender and receiver BlacklistEntry PDAs as extra accounts
- `transfer_hook`: Checks if sender or receiver BlacklistEntry PDA exists. If found, rejects the transfer.
- Implements Anchor fallback for SPL Transfer Hook Interface compatibility

## TypeScript SDK Design

```typescript
export enum Presets { SSS_1 = "sss-1", SSS_2 = "sss-2" }

export class SolanaStablecoin {
  private constructor(/* internal */) {}

  static async create(connection, config): Promise<SolanaStablecoin>;
  static async load(connection, mint): Promise<SolanaStablecoin>;

  // Core operations
  async mint(params: MintParams): Promise<TransactionSignature>;
  async burn(params: BurnParams): Promise<TransactionSignature>;
  async freezeAccount(address: PublicKey): Promise<TransactionSignature>;
  async thawAccount(address: PublicKey): Promise<TransactionSignature>;
  async pause(): Promise<TransactionSignature>;
  async unpause(): Promise<TransactionSignature>;
  async getTotalSupply(): Promise<BN>;
  async getConfig(): Promise<StablecoinConfig>;

  // Compliance module (SSS-2 returns real module; SSS-1 throws)
  get compliance(): ComplianceModule;
}

export class ComplianceModule {
  async blacklistAdd(address, reason): Promise<TransactionSignature>;
  async blacklistRemove(address): Promise<TransactionSignature>;
  async seize(from, to): Promise<TransactionSignature>;
  async isBlacklisted(address): Promise<boolean>;
}
```

## Rust CLI Design

Uses clap v4 derive API. Commands match requirements exactly:

```
sss-token init --preset sss-1|sss-2 | --custom config.toml
sss-token mint <recipient> <amount>
sss-token burn <amount>
sss-token freeze <address>
sss-token thaw <address>
sss-token pause | unpause
sss-token status | supply
sss-token blacklist add|remove <address> [--reason "..."]
sss-token seize <address> --to <treasury>
sss-token minters list|add|remove
sss-token holders [--min-balance <amount>]
sss-token audit-log [--action <type>]
```

## Backend Services (Rust/Axum)

Four containerized microservices:

1. **mint-burn**: REST API for mint/burn lifecycle (request → verify → execute → log)
2. **compliance**: Blacklist management, sanctions screening integration, audit trail export
3. **indexer**: WebSocket subscription to on-chain events, SQLite state tracking
4. **webhook**: Configurable event notifications with exponential backoff retry

Shared: tracing for structured logging, /health endpoints, .env config, SQLite via sqlx.

## Testing Strategy

| Type | Tool | Scope |
|------|------|-------|
| Rust unit tests | cargo test | math, state, error codes |
| TS SDK unit tests | mocha | SDK functions, PDA derivation |
| SSS-1 integration | anchor test | init → mint → transfer → freeze → thaw |
| SSS-2 integration | anchor test | init → mint → transfer → blacklist → seize |
| Compliance tests | anchor test | blacklist CRUD, seize, feature gating |
| Fuzz tests | Trident | state invariants, edge cases, overflow |
| Devnet smoke tests | scripts/ | Real network deployment + example operations |

## Bonus Features

1. **SSS-3 Private Stablecoin**: Documentation-only PoC. Confidential Transfers are currently disabled on mainnet/devnet. Provides architecture spec and future implementation guide.
2. **Oracle Integration Module**: Switchboard oracle program for non-USD pegs (EUR, BRL, CPI-indexed). Separate program for mint/redeem pricing.
3. **Admin TUI**: ratatui-based terminal UI with real-time supply dashboard, operation logs, blacklist management, minter quota monitoring.
4. **Frontend Example**: Next.js app using @stbr/sss-token SDK for stablecoin creation and management demo.

## Technical Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Transfer Hook CPI complexity | High | Early prototype, reference PYUSD and Civic implementations |
| Anchor + Token-2022 extension init | High | Use raw invoke_signed (SVS validated pattern) |
| Permanent Delegate key security | High | Enforce multisig, restrict seizer role |
| Confidential Transfers disabled | Medium | SSS-3 as documentation PoC only |
| ExtraAccountMetas PDA size limits | Medium | Calculate exact account requirements |
| Devnet deployment instability | Low | Retry logic + multiple RPC providers |

## Extension Compatibility

- Transfer Hook + Permanent Delegate: COMPATIBLE
- Transfer Hook + Confidential Transfers: INCOMPATIBLE (SSS-2 and SSS-3 are mutually exclusive)
- DefaultAccountState + Freeze Authority: COMPATIBLE (but never revoke freeze if default=frozen)

## GENIUS Act Compliance (SSS-2)

SSS-2 satisfies all GENIUS Act technical requirements:
- Freeze: via freeze authority
- Seize: via permanent delegate (transferChecked)
- Burn: via permanent delegate (burnChecked)
- Block transactions: via transfer hook blacklist enforcement
