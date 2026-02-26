# SSS-1: Minimal Stablecoin Standard

## Overview

SSS-1 is the minimal stablecoin preset for Solana. It provides the core building blocks for issuing and managing a fiat-backed or asset-backed token on Token-2022 without regulatory compliance features.

SSS-1 is suitable for:
- Internal ecosystem settlement tokens
- DAO treasury tokens
- Institutional tokens with off-chain compliance
- Development and testing environments
- Custom stablecoins that handle compliance at the application layer

## Token-2022 Extensions

SSS-1 activates the following extensions on the mint account:

| Extension | Purpose |
|-----------|---------|
| MintAuthority | Set to StablecoinConfig PDA. Only the program can mint tokens. |
| FreezeAuthority | Set to StablecoinConfig PDA. Enables per-account freezing. |
| MetadataPointer | Self-referencing. Points to the mint account itself for metadata. |
| TokenMetadata | On-chain name, symbol, and URI stored directly on the mint. |

Extensions NOT used by SSS-1:

| Extension | Why Not |
|-----------|---------|
| PermanentDelegate | Not needed — no asset seizure capability |
| TransferHook | Not needed — no on-chain blacklist enforcement |
| DefaultAccountState | Not needed — accounts start unfrozen |

## Initialization Parameters

```rust
InitializeParams {
    name: "Superteam USD",
    symbol: "stUSD",
    uri: "https://example.com/metadata.json",
    decimals: 6,
    enable_permanent_delegate: false,  // SSS-1 default
    enable_transfer_hook: false,       // SSS-1 default
    default_account_frozen: false,     // SSS-1 default
}
```

## Instruction Reference

| Instruction | Authority | Description |
|-------------|-----------|-------------|
| `initialize` | authority | Create mint with SSS-1 extensions, init config and roles |
| `mint` | minter | Mint tokens to recipient (quota-enforced) |
| `burn` | burner | Burn tokens from caller's account |
| `freeze_account` | authority/pauser | Freeze a token account |
| `thaw_account` | authority/pauser | Thaw a frozen account |
| `pause` | pauser | Global pause (blocks mint/burn) |
| `unpause` | pauser | Resume operations |
| `update_minter` | authority | Add/remove/update minter quotas |
| `update_roles` | authority | Change role assignments |
| `transfer_authority` | authority | Transfer master authority (two-step) |

SSS-2 instructions (`add_to_blacklist`, `remove_from_blacklist`, `seize`) are rejected with `ComplianceNotEnabled` error (6006) when called on an SSS-1 stablecoin.

## Account Layout

### PDAs Created

```
┌─────────────────────────────────────────────┐
│  StablecoinConfig                            │
│  Seeds: ["stablecoin", mint.key()]           │
│  Size: ~460 bytes                            │
│  Rent: ~0.0046 SOL                           │
├─────────────────────────────────────────────┤
│  RoleConfig                                  │
│  Seeds: ["roles", stablecoin_config.key()]   │
│  Size: ~880 bytes                            │
│  Rent: ~0.0088 SOL                           │
└─────────────────────────────────────────────┘
```

Total initialization cost: ~0.02 SOL (including mint account).

## Role Model

```
master_authority
├── pauser         → pause, unpause, freeze, thaw
├── minters[]      → mint (per-minter quota)
└── burners[]      → burn
```

All roles default to the initializing authority. Use `update_roles` and `update_minter` to assign separate keys.

## Security Considerations

### Mint Authority as PDA

The mint authority is the StablecoinConfig PDA, not a user-controlled key. This means:
- No single key can mint tokens outside the program
- Minting always enforces quota checks
- The program logic cannot be bypassed

### Freeze Authority

The freeze authority is also the StablecoinConfig PDA. This allows:
- Authority and pauser to freeze individual accounts via the program
- Emergency response without revoking freeze authority
- No external freeze authority that could be compromised

### Per-Minter Quotas

Each minter has an independent quota tracked on-chain:
- Maximum 10 minters (configurable constant)
- Each minter has: `address`, `quota` (max allowed), `minted` (cumulative)
- `remaining_quota = quota - minted`
- Quota exhaustion requires authority intervention to increase

### Global Pause

The pause mechanism is a boolean flag on StablecoinConfig. When paused:
- `mint` → rejected with `Paused` error
- `burn` → rejected with `Paused` error
- `freeze/thaw` → still allowed (for emergency response)
- `update_*` → still allowed (for remediation)
- `transfer_authority` → still allowed

## Example Deployment Walkthrough

### 1. Initialize

```bash
sss-token init --preset sss-1 \
  --name "Superteam USD" \
  --symbol "stUSD" \
  --decimals 6
```

### 2. Configure Roles

```bash
# Add a minter with 1M token quota
sss-token minters add <MINTER_A> --quota 1000000000000

# Add a second minter with 500K quota
sss-token minters add <MINTER_B> --quota 500000000000

# Set a separate pauser
sss-token update-roles --pauser <PAUSER_KEY>
```

### 3. Mint Tokens

```bash
# Minter A mints 100K tokens to a recipient
sss-token mint <RECIPIENT> 100000000000

# Check remaining quota
sss-token minters list
```

### 4. Normal Operations

```bash
# Users transfer tokens using standard spl-token transfer_checked
# No transfer hook — standard Token-2022 transfers

# Burn tokens
sss-token burn 50000000000

# Check supply
sss-token supply
```

### 5. Emergency Response

```bash
# Freeze a suspicious account
sss-token freeze <ACCOUNT>

# If situation escalates, pause everything
sss-token pause

# After investigation, resume
sss-token thaw <ACCOUNT>
sss-token unpause
```

## SDK Usage

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";

// Create
const coin = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_1,
  name: "Superteam USD",
  symbol: "stUSD",
  decimals: 6,
  authority: authority.publicKey,
});

// Compliance check
try {
  await coin.compliance.blacklistAdd(address, "test");
} catch (err) {
  // Error: ComplianceNotEnabled — expected for SSS-1
}
```

## Comparison with SSS-2

| Feature | SSS-1 | SSS-2 |
|---------|-------|-------|
| Mint/Burn with quotas | Yes | Yes |
| Freeze/Thaw | Yes | Yes |
| Pause/Unpause | Yes | Yes |
| On-chain metadata | Yes | Yes |
| Permanent Delegate | No | Yes |
| Transfer Hook | No | Yes |
| Blacklist | No | Yes |
| Seize | No | Yes |
| GENIUS Act compliant | No | Yes |
| Init cost | ~0.02 SOL | ~0.04 SOL |
| Transfer overhead | None | Hook CPI (~5K CU) |

Choose SSS-1 when you need a simple, efficient stablecoin without on-chain compliance requirements. Choose SSS-2 when regulatory compliance (freeze, seize, block) must be enforced at the protocol level.
