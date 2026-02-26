# SSS-2: Compliant Stablecoin Standard

## Overview

SSS-2 is the regulatory-compliant stablecoin preset for Solana. It extends SSS-1 with on-chain enforcement mechanisms required by laws like the GENIUS Act (S. 1582), enabling issuers to freeze, seize, burn, and block transactions at the protocol level.

SSS-2 is designed for:
- USDC/USDT-class regulated payment stablecoins
- Bank-issued tokens under federal/state regulation
- Stablecoins requiring OFAC sanctions compliance
- Tokens that must satisfy the GENIUS Act's technical requirements

## Token-2022 Extensions

SSS-2 activates all SSS-1 extensions plus compliance extensions:

| Extension | Purpose |
|-----------|---------|
| MintAuthority | PDA-controlled minting with per-minter quotas |
| FreezeAuthority | Per-account freezing for sanctions response |
| MetadataPointer | Self-referencing on-chain metadata |
| TokenMetadata | Name, symbol, URI stored on mint |
| **PermanentDelegate** | Allows seizing/burning tokens from any account without holder consent |
| **TransferHook** | Invokes sss-transfer-hook on every transfer to enforce blacklist |
| **DefaultAccountState** | (Optional) New token accounts start frozen — requires explicit approval |

## GENIUS Act Compliance Mapping

The Guiding and Establishing National Innovation for U.S. Stablecoins (GENIUS) Act (S. 1582) requires issuers to maintain certain technical controls. SSS-2 satisfies each requirement:

| GENIUS Act Requirement | SSS-2 Implementation | On-Chain Enforcement |
|-----------------------|----------------------|---------------------|
| Freeze assets on court order | `freeze_account` instruction via FreezeAuthority | Yes — frozen accounts cannot transact |
| Seize assets per law enforcement | `seize` instruction via PermanentDelegate | Yes — transfers tokens without holder signature |
| Burn confiscated tokens | `burn` via PermanentDelegate authority | Yes — destroys tokens from any account |
| Block sanctioned transactions | TransferHook + BlacklistEntry PDAs | Yes — every transfer checked against blacklist |
| Maintain audit trail | On-chain events (Anchor `emit!`) | Yes — all actions emit structured events |
| Role separation | RoleConfig with separate blacklister/seizer keys | Yes — enforced at program level |

## Blacklist Enforcement Flow

```
┌────────────────────────────────────────────────────────────────┐
│                    Blacklist Enforcement                        │
│                                                                │
│  1. Blacklister adds address to blacklist                      │
│     └─> Creates BlacklistEntry PDA:                            │
│         seeds = ["blacklist", mint, sanctioned_address]        │
│                                                                │
│  2. Sanctioned user attempts transfer                          │
│     └─> Token-2022 calls transfer_checked                     │
│         └─> Token-2022 reads TransferHook extension            │
│             └─> Resolves ExtraAccountMetaList                  │
│                 └─> Derives sender & receiver BlacklistEntry   │
│                     └─> CPIs into sss-transfer-hook            │
│                                                                │
│  3. Transfer Hook checks blacklist                             │
│     ├─> Sender BlacklistEntry exists?                          │
│     │   YES → Error: BlacklistedSender (transfer blocked)      │
│     │   NO  → continue                                         │
│     └─> Receiver BlacklistEntry exists?                        │
│         YES → Error: BlacklistedReceiver (transfer blocked)    │
│         NO  → Ok (transfer proceeds)                           │
└────────────────────────────────────────────────────────────────┘
```

Key properties:
- Enforcement is **automatic** — no reliance on frontend filtering
- Enforcement is **bidirectional** — blocks both sending and receiving
- Enforcement is **immutable** — the transfer hook program cannot be swapped after initialization
- Enforcement is **gas-efficient** — single PDA existence check (~5K compute units)

## Seize Procedure

Step-by-step process for asset seizure:

```
Step 1: Identify target
        └─> Law enforcement request with target address

Step 2: Blacklist the address (prevents further movement)
        └─> sss-token blacklist add <ADDRESS> --reason "Court Order #12345"
        └─> Creates BlacklistEntry PDA
        └─> All transfers blocked immediately

Step 3: Seize tokens
        └─> sss-token seize <ADDRESS> --to <TREASURY>
        └─> Program uses PermanentDelegate to transfer_checked
        └─> Tokens moved to treasury WITHOUT target's signature
        └─> TokensSeized event emitted

Step 4: (Optional) Burn seized tokens
        └─> sss-token burn <amount> (from treasury account)
        └─> Permanently destroys confiscated tokens

Step 5: Document
        └─> sss-token audit-log --action seize
        └─> Export on-chain evidence for legal proceedings
```

### Seize Implementation Detail

The `seize` instruction uses the StablecoinConfig PDA as the permanent delegate:

```rust
// Permanent delegate = StablecoinConfig PDA
// Signer seeds: ["stablecoin", mint.key(), &[bump]]
token_2022::transfer_checked(
    CpiContext::new_with_signer(
        token_program,
        TransferChecked {
            from: target_token_account,
            to: treasury_token_account,
            mint: mint,
            authority: stablecoin_config,  // PDA as permanent delegate
        },
        signer_seeds,
    ),
    amount,
    decimals,
)?;
```

## Instruction Reference

All SSS-1 instructions plus:

| Instruction | Authority | Description |
|-------------|-----------|-------------|
| `add_to_blacklist` | blacklister | Create BlacklistEntry PDA for an address |
| `remove_from_blacklist` | blacklister | Close BlacklistEntry PDA, reclaim rent |
| `seize` | seizer | Transfer tokens from target to treasury via permanent delegate |

### Feature Gating

SSS-2 instructions check compliance flags before execution:

```rust
require!(
    ctx.accounts.stablecoin_config.enable_transfer_hook
        && ctx.accounts.stablecoin_config.enable_permanent_delegate,
    StablecoinError::ComplianceNotEnabled
);
```

## Account Layout

### Additional PDAs (beyond SSS-1)

```
┌─────────────────────────────────────────────────┐
│  BlacklistEntry (per blacklisted address)        │
│  Seeds: ["blacklist", mint.key(), address.key()] │
│  Size: ~245 bytes                                │
│  Rent: ~0.0025 SOL per entry                     │
├─────────────────────────────────────────────────┤
│  ExtraAccountMetaList (one per mint)             │
│  Seeds: ["extra-account-metas", mint.key()]      │
│  Program: sss-transfer-hook                      │
│  Stores: sender/receiver BlacklistEntry PDA      │
│          derivation instructions                  │
└─────────────────────────────────────────────────┘
```

## Audit Trail Format

All SSS-2 compliance events are emitted as Anchor events and can be indexed:

### Event: AddressBlacklisted

```json
{
  "event": "AddressBlacklisted",
  "data": {
    "mint": "7Xf3...kP9",
    "address": "2Rf4...jL6",
    "reason": "OFAC SDN List - Entity XYZ",
    "by": "4Gh2...mN8"
  },
  "slot": 123456789,
  "blockTime": 1740412800,
  "signature": "5Kj2...txSig"
}
```

### Event: TokensSeized

```json
{
  "event": "TokensSeized",
  "data": {
    "mint": "7Xf3...kP9",
    "from": "2Rf4...jL6",
    "to": "9Qw1...tR5",
    "amount": 50000000000,
    "by": "8Mn3...wP4"
  },
  "slot": 123456800,
  "blockTime": 1740412850,
  "signature": "7Lm4...txSig"
}
```

### Audit Trail Export

```bash
# Export all compliance events as JSON
sss-token audit-log --format json > audit-trail.json

# Filter by action
sss-token audit-log --action blacklist --format json
sss-token audit-log --action seize --format json

# Filter by date range
sss-token audit-log --after 2026-01-01 --before 2026-02-01
```

## Comparison with PYUSD Architecture

| Feature | SSS-2 | PYUSD |
|---------|-------|-------|
| Blockchain | Solana (Token-2022) | Ethereum (ERC-20) + Solana (Token-2022) |
| Freeze mechanism | FreezeAuthority on PDA | FreezeAuthority on multisig |
| Blacklist storage | On-chain PDAs per address | On-chain mapping (ETH) / Transfer Hook (SOL) |
| Seize mechanism | PermanentDelegate PDA | PermanentDelegate (SOL) / admin transfer (ETH) |
| Transfer blocking | Transfer Hook program | Transfer Hook program (SOL) / require check (ETH) |
| Role separation | Configurable via RoleConfig | Hardcoded in contract |
| Open source SDK | Yes (@stbr/sss-token) | Partial |
| Multi-preset | Yes (SSS-1 for minimal, SSS-2 for compliant) | Single design |
| Audit trail | Anchor events + indexer | Contract events |

SSS-2 differentiates by providing a modular, SDK-first approach where the same program supports both minimal (SSS-1) and compliant (SSS-2) configurations.

## Security Considerations

### Permanent Delegate Risks

The permanent delegate is the most powerful capability in SSS-2. It allows transferring tokens from ANY token account without the holder's consent.

Mitigations:
- The delegate is the StablecoinConfig PDA, not a user key
- Only the `seizer` role can trigger seizure through the program
- Every seizure emits an auditable `TokensSeized` event
- Recommended: seizer key should be a multisig (Squads 3-of-5)

### Transfer Hook Immutability

Once a TransferHook extension is initialized on a mint, the hook program cannot be changed. This ensures:
- Blacklist enforcement cannot be bypassed by changing the hook
- Upgrade path requires token migration (by design, for regulatory certainty)

### BlacklistEntry PDA Rent

Each blacklist entry costs ~0.0025 SOL in rent. Removing an address from the blacklist closes the PDA and reclaims rent. Issuers should budget for worst-case blacklist size.

## Example: Full SSS-2 Lifecycle

```bash
# 1. Initialize SSS-2 stablecoin
sss-token init --preset sss-2 \
  --name "Regulated USD" \
  --symbol "rUSD" \
  --decimals 6

# 2. Configure compliance roles
sss-token update-roles \
  --blacklister <COMPLIANCE_TEAM_KEY> \
  --seizer <LEGAL_MULTISIG_KEY>

# 3. Add minters
sss-token minters add <MINTER_KEY> --quota 10000000000000

# 4. Mint tokens
sss-token mint <USER_A> 1000000000

# 5. Normal transfers work (neither party blacklisted)
# Users transfer via standard spl-token commands

# 6. Sanctions alert: blacklist an address
sss-token blacklist add <BAD_ACTOR> --reason "OFAC SDN List"

# 7. Bad actor's transfers are now blocked automatically
# Any transfer_checked involving BAD_ACTOR will fail

# 8. Court order: seize assets
sss-token seize <BAD_ACTOR> --to <TREASURY>

# 9. Export audit trail for legal
sss-token audit-log --format json > evidence.json

# 10. Case resolved: unblacklist
sss-token blacklist remove <BAD_ACTOR>
```
