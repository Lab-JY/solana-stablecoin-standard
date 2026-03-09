# Solana Stablecoin Standard (SSS)

A modular stablecoin SDK for Solana built on Token-2022 extensions. Two opinionated presets — **SSS-1** (minimal) and **SSS-2** (regulatory-compliant) — plus a TypeScript SDK, Rust CLI, and backend services.

Built by [Superteam Brazil](https://superteam.fun) for the Solana ecosystem.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Applications                              │
│   Next.js Frontend  │  Rust CLI  │  Admin TUI  │  Backend API   │
└──────────┬──────────┴─────┬──────┴──────┬──────┴───────┬────────┘
           │                │             │              │
┌──────────▼────────────────▼─────────────▼──────────────▼────────┐
│                    TypeScript SDK (@stbr/sss-token)              │
│  ┌──────────────┐  ┌─────────────────┐  ┌────────────────────┐  │
│  │ SolanaStable │  │ ComplianceModule│  │  PDA Derivation    │  │
│  │    coin      │  │   (SSS-2 only)  │  │  & Presets         │  │
│  └──────┬───────┘  └────────┬────────┘  └─────────┬──────────┘  │
└─────────┼──────────────────┼──────────────────────┼─────────────┘
          │                  │                      │
┌─────────▼──────────────────▼──────────────────────▼─────────────┐
│                   On-Chain Programs (Anchor 0.31.x)              │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                    sss-token Program                        │  │
│  │  ┌─────────────────────┐  ┌─────────────────────────────┐  │  │
│  │  │  SSS-1 (Minimal)    │  │  SSS-2 (Compliant)          │  │  │
│  │  │  - MintAuthority    │  │  - All SSS-1 extensions     │  │  │
│  │  │  - FreezeAuthority  │  │  + PermanentDelegate        │  │  │
│  │  │  - MetadataPointer  │  │  + TransferHook             │  │  │
│  │  │  - TokenMetadata    │  │  + DefaultAccountState      │  │  │
│  │  └─────────────────────┘  └─────────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              sss-transfer-hook Program                      │  │
│  │  Enforces blacklist checks on every token transfer (SSS-2) │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│                       Solana Token-2022                           │
└──────────────────────────────────────────────────────────────────┘
```

## SSS-1 vs SSS-2 Comparison

| Feature | SSS-1 (Minimal) | SSS-2 (Compliant) |
|---------|------------------|--------------------|
| Target Use Case | DAO treasuries, ecosystem settlement | USDC/USDT-class regulated tokens |
| Mint/Burn | Per-minter quotas | Per-minter quotas |
| Freeze/Thaw | Individual accounts | Individual accounts |
| Pause/Unpause | Global circuit breaker | Global circuit breaker |
| Token Metadata | On-chain via Token-2022 | On-chain via Token-2022 |
| Permanent Delegate | No | Yes (seize/burn from any account) |
| Transfer Hook | No | Yes (blacklist enforcement) |
| Default Account State | Normal | Configurable (frozen by default) |
| Blacklist | No | On-chain OFAC-compatible |
| Seize Assets | No | Yes (via permanent delegate) |
| GENIUS Act Compliant | No | Yes |
| Role Separation | authority, pauser, minters, burners | + blacklister, seizer |

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) 1.18+
- [Anchor](https://www.anchor-lang.com/docs/installation) 0.31.x
- [Node.js](https://nodejs.org/) 18+
- [Yarn](https://yarnpkg.com/) 1.22+

### Build

```bash
# Clone the repository
git clone https://github.com/solanabr/solana-stablecoin-standard.git
cd solana-stablecoin-standard

# Install dependencies
yarn install

# Build on-chain programs
anchor build

# Build Rust CLI
cargo build -p sss-token-cli

# Build TypeScript SDK
yarn workspace @stbr/sss-token build
```

### Deploy (Localnet)

```bash
# Start local validator with Token-2022
solana-test-validator --reset

# Deploy programs
anchor deploy

# Run integration tests
anchor test
```

### CLI Quick Start — SSS-1 (Minimal Stablecoin)

```bash
# Initialize an SSS-1 stablecoin
sss-token init --preset sss-1 \
  --name "Superteam USD" \
  --symbol "stUSD" \
  --decimals 6

# Add a minter with 1M token quota
sss-token minters add <MINTER_PUBKEY> --quota 1000000000000

# Mint tokens
sss-token mint <RECIPIENT_PUBKEY> 1000000000  # 1,000 tokens (6 decimals)

# Check supply
sss-token supply

# Freeze an account
sss-token freeze <ACCOUNT_PUBKEY>

# Emergency pause
sss-token pause
```

### CLI Quick Start — SSS-2 (Compliant Stablecoin)

```bash
# Initialize an SSS-2 stablecoin with all compliance features
sss-token init --preset sss-2 \
  --name "Regulated USD" \
  --symbol "rUSD" \
  --decimals 6

# Blacklist a sanctioned address
sss-token blacklist add <ADDRESS> --reason "OFAC SDN List"

# Seize tokens from a blacklisted address
sss-token seize <ADDRESS> --to <TREASURY_PUBKEY>

# Export audit trail
sss-token audit-log --action blacklist
```

### SDK Quick Start — TypeScript

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";
import { Connection, Keypair } from "@solana/web3.js";

const connection = new Connection("http://localhost:8899");
const authority = Keypair.generate();

// Create an SSS-1 stablecoin
const stablecoin = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_1,
  name: "Superteam USD",
  symbol: "stUSD",
  decimals: 6,
  authority: authority.publicKey,
});

// Mint tokens
await stablecoin.mint({
  recipient: recipientPubkey,
  amount: new BN(1_000_000_000), // 1,000 tokens
  minter: minterKeypair,
});

// Get total supply
const supply = await stablecoin.getTotalSupply();
console.log("Supply:", supply.toString());

// SSS-2: Compliance operations
const sss2 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "Regulated USD",
  symbol: "rUSD",
  decimals: 6,
  authority: authority.publicKey,
});

// Blacklist an address
await sss2.compliance.blacklistAdd(suspectAddress, "OFAC SDN List");

// Seize tokens
await sss2.compliance.seize(fromAddress, treasuryAddress);
```

## CLI Command Reference

| Command | Description | Preset |
|---------|-------------|--------|
| `sss-token init --preset sss-1\|sss-2 \| --custom config.toml` | Initialize a new stablecoin | ALL |
| `sss-token mint <recipient> <amount>` | Mint tokens to a recipient | ALL |
| `sss-token burn <amount>` | Burn tokens | ALL |
| `sss-token freeze <address>` | Freeze a token account | ALL |
| `sss-token thaw <address>` | Thaw a frozen token account | ALL |
| `sss-token pause` | Pause all operations globally | ALL |
| `sss-token unpause` | Resume operations | ALL |
| `sss-token status` | Show stablecoin configuration and state | ALL |
| `sss-token supply` | Show current token supply | ALL |
| `sss-token minters list` | List all minters and quotas | ALL |
| `sss-token minters add <address> --quota <amount>` | Add a minter | ALL |
| `sss-token minters remove <address>` | Remove a minter | ALL |
| `sss-token holders [--min-balance <amount>]` | List token holders | ALL |
| `sss-token blacklist add <address> [--reason "..."]` | Blacklist an address | SSS-2 |
| `sss-token blacklist remove <address>` | Remove from blacklist | SSS-2 |
| `sss-token seize <address> --to <treasury>` | Seize tokens via permanent delegate | SSS-2 |
| `sss-token audit-log [--action <type>]` | Export audit trail | SSS-2 |

## PDA Derivation

| Account | Seeds | Description |
|---------|-------|-------------|
| `StablecoinConfig` | `["stablecoin", mint]` | Main configuration and state |
| `RoleConfig` | `["roles", stablecoin_config]` | Role assignments and minter quotas |
| `BlacklistEntry` | `["blacklist", mint, address]` | Per-address blacklist record (SSS-2) |
| `ExtraAccountMetaList` | `["extra-account-metas", mint]` (transfer hook program) | Transfer Hook extra accounts |

## Instruction Reference

| Instruction | Authority | Description |
|-------------|-----------|-------------|
| `initialize` | authority | Create Token-2022 mint, configure extensions, init state |
| `mint` | minter | Mint tokens (enforces per-minter quota) |
| `burn` | burner | Burn tokens from caller's account |
| `freeze_account` | authority / pauser | Freeze a specific token account |
| `thaw_account` | authority / pauser | Thaw a frozen token account |
| `pause` | pauser | Set global pause flag — blocks mint/burn/transfer |
| `unpause` | pauser | Clear global pause flag |
| `update_minter` | authority | Add, remove, or update minter quota |
| `update_roles` | authority | Update pauser, blacklister, seizer, burners |
| `transfer_authority` | authority | Transfer master authority (two-step) |
| `add_to_blacklist` | blacklister | Create BlacklistEntry PDA (SSS-2 only) |
| `remove_from_blacklist` | blacklister | Close BlacklistEntry PDA (SSS-2 only) |
| `seize` | seizer | Transfer tokens via permanent delegate (SSS-2 only) |

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| 6000 | `Paused` | The stablecoin is currently paused |
| 6001 | `Unauthorized` | Caller does not have the required role |
| 6002 | `MinterQuotaExceeded` | Mint amount exceeds minter's remaining quota |
| 6003 | `MinterNotFound` | Minter address not in the minters list |
| 6004 | `MaxMintersReached` | Maximum of 10 minters already configured |
| 6005 | `MaxBurnersReached` | Maximum of 10 burners already configured |
| 6006 | `ComplianceNotEnabled` | SSS-2 instruction called on SSS-1 stablecoin |
| 6007 | `AlreadyBlacklisted` | Address already has a BlacklistEntry |
| 6008 | `NotBlacklisted` | Address is not blacklisted |
| 6009 | `NameTooLong` | Name exceeds 32 bytes |
| 6010 | `SymbolTooLong` | Symbol exceeds 10 bytes |
| 6011 | `UriTooLong` | URI exceeds 200 bytes |
| 6012 | `ReasonTooLong` | Blacklist reason exceeds 128 bytes |
| 6013 | `MathOverflow` | Arithmetic overflow in supply tracking |
| 6014 | `AccountFrozen` | Target token account is frozen |
| 6015 | `InvalidAuthority` | Provided authority does not match config |
| 6016 | `MinterAlreadyExists` | Minter address already in the list |
| 6017 | `BurnerAlreadyExists` | Burner address already in the list |
| 6018 | `BurnerNotFound` | Burner address not in the list |

## Events

| Event | Fields | Description |
|-------|--------|-------------|
| `StablecoinInitialized` | mint, authority, name, symbol, decimals, preset | Emitted on stablecoin creation |
| `TokensMinted` | mint, recipient, amount, minter | Emitted on each mint operation |
| `TokensBurned` | mint, amount, burner | Emitted on each burn operation |
| `AccountFrozen` | mint, account, by | Emitted when account is frozen |
| `AccountThawed` | mint, account, by | Emitted when account is thawed |
| `StablecoinPaused` | mint, by | Emitted on global pause |
| `StablecoinUnpaused` | mint, by | Emitted on unpause |
| `MinterUpdated` | mint, minter, quota, action | Emitted on minter add/remove/update |
| `RolesUpdated` | mint, role, address, by | Emitted on role assignment changes |
| `AuthorityTransferred` | mint, old_authority, new_authority | Emitted on authority transfer |
| `AddressBlacklisted` | mint, address, reason, by | Emitted when address is blacklisted (SSS-2) |
| `AddressUnblacklisted` | mint, address, by | Emitted when address is removed from blacklist (SSS-2) |
| `TokensSeized` | mint, from, to, amount, by | Emitted on asset seizure (SSS-2) |

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/ARCHITECTURE.md) | Three-layer architecture, account structures, data flows |
| [SDK Reference](docs/SDK.md) | TypeScript SDK installation, API reference, examples |
| [Operations Guide](docs/OPERATIONS.md) | Operator runbook for all mint/burn/compliance operations |
| [SSS-1 Specification](docs/SSS-1.md) | Minimal Stablecoin Standard spec and walkthrough |
| [SSS-2 Specification](docs/SSS-2.md) | Compliant Stablecoin Standard spec with GENIUS Act mapping |
| [SSS-3 Specification](docs/SSS-3.md) | Private Stablecoin PoC with Confidential Transfers |
| [Compliance Guide](docs/COMPLIANCE.md) | GENIUS Act compliance, audit trails, sanctions screening |
| [API Reference](docs/API.md) | Backend services REST API documentation |
| [Bounty Traceability](docs/REQUIREMENTS_TRACEABILITY.md) | Requirement-by-requirement submission status |

## Project Structure

```
solana-stablecoin-standard/
├── programs/
│   ├── sss-token/              # Main Anchor program (SSS-1 + SSS-2)
│   └── sss-transfer-hook/      # Transfer Hook for blacklist enforcement
├── sdk/core/                   # @stbr/sss-token TypeScript SDK
├── cli/                        # Rust CLI (sss-token)
├── services/                   # Rust backend services (Axum)
│   ├── shared/                 # Shared library (auth, rate limiting, metrics, middleware)
│   ├── mint-burn/              # Mint/Burn lifecycle API
│   ├── compliance/             # Compliance & audit trail API
│   ├── indexer/                # On-chain event indexer
│   ├── webhook/                # Webhook notifications
│   ├── docker-compose.yml
│   └── .env.example            # Environment variable template
├── tui/                        # Admin TUI (ratatui)
├── app/                        # Next.js frontend example
├── tests/                      # Integration tests
├── trident-tests/              # Fuzz tests (Trident)
├── scripts/                    # Deployment & smoke test scripts
└── docs/                       # Documentation
```

## License

[MIT](LICENSE) - Copyright (c) 2026 Superteam Brazil
