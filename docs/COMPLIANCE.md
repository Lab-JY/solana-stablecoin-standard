# Compliance Guide

## Overview

This document describes how the Solana Stablecoin Standard (SSS-2 preset) satisfies regulatory requirements for payment stablecoins, with specific focus on the GENIUS Act (S. 1582).

## GENIUS Act (S. 1582) Requirements

The Guiding and Establishing National Innovation for U.S. Stablecoins Act requires payment stablecoin issuers to maintain technical controls for law enforcement compliance. Below is a detailed mapping of each requirement to SSS-2's implementation.

### Requirement 1: Freeze Assets

> Issuers must be able to freeze tokens held by specific addresses upon court order or regulatory directive.

**SSS-2 Implementation:**

| Mechanism | Detail |
|-----------|--------|
| Extension | FreezeAuthority (Token-2022 native) |
| Authority | StablecoinConfig PDA |
| Instruction | `freeze_account` |
| Role | authority or pauser |
| Effect | Frozen accounts cannot send or receive tokens |
| Reversible | Yes, via `thaw_account` |

```bash
# Freeze an account
sss-token freeze <ADDRESS>

# Verify frozen status
sss-token holders  # shows freeze status per account
```

### Requirement 2: Seize Assets

> Issuers must be able to transfer tokens from a holder's account to a designated address per law enforcement order.

**SSS-2 Implementation:**

| Mechanism | Detail |
|-----------|--------|
| Extension | PermanentDelegate (Token-2022) |
| Delegate | StablecoinConfig PDA |
| Instruction | `seize` |
| Role | seizer |
| Effect | Transfers all tokens to treasury without holder's signature |
| Reversible | No (by design — legal finality) |

```bash
# Seize tokens
sss-token seize <TARGET> --to <TREASURY>
```

### Requirement 3: Burn Confiscated Tokens

> Issuers must be able to destroy confiscated tokens.

**SSS-2 Implementation:**

Burning is performed from the treasury account after seizure:

```bash
# Burn tokens from treasury
sss-token burn <AMOUNT>
```

### Requirement 4: Block Sanctioned Transactions

> Issuers must prevent sanctioned addresses from transacting.

**SSS-2 Implementation:**

| Mechanism | Detail |
|-----------|--------|
| Extension | TransferHook (Token-2022) |
| Hook Program | sss-transfer-hook |
| Storage | BlacklistEntry PDAs |
| Enforcement | Automatic on every `transfer_checked` |
| Direction | Bidirectional (blocks sending AND receiving) |

```bash
# Block an address
sss-token blacklist add <ADDRESS> --reason "OFAC SDN List"

# All future transfers involving this address are rejected
# No frontend cooperation needed — enforced at protocol level
```

### Requirement 5: Maintain Audit Trail

> Issuers must maintain records of all compliance actions.

**SSS-2 Implementation:**

All compliance actions emit Anchor events that are recorded on-chain:

| Event | When Emitted |
|-------|-------------|
| `AddressBlacklisted` | Address added to blacklist |
| `AddressUnblacklisted` | Address removed from blacklist |
| `TokensSeized` | Assets transferred via permanent delegate |
| `AccountFrozen` | Account frozen |
| `AccountThawed` | Account unfrozen |
| `StablecoinPaused` | Global pause activated |
| `StablecoinUnpaused` | Global pause deactivated |

```bash
# Export full audit trail
sss-token audit-log --format json > audit-$(date +%Y%m%d).json

# The indexer service also stores events in SQLite for querying
```

### Requirement 6: Role Separation

> Access to compliance functions should be restricted and separated.

**SSS-2 Implementation:**

```
┌───────────────────────────────────────────────────┐
│  Role Separation Matrix                            │
│                                                    │
│  Role            │ Can Do         │ Cannot Do      │
│  ─────────────────┼────────────────┼──────────────  │
│  master_authority │ Assign roles   │ Mint, blacklist│
│  pauser           │ Pause, freeze  │ Mint, seize    │
│  minter           │ Mint (quota)   │ Freeze, seize  │
│  burner           │ Burn           │ Mint, freeze   │
│  blacklister      │ Blacklist mgmt │ Seize, mint    │
│  seizer           │ Seize assets   │ Blacklist, mint│
└───────────────────────────────────────────────────┘
```

No single key can perform all compliance operations. Each role is a separate public key enforced on-chain.

## Role Separation Best Practices

### Recommended Key Architecture

```
┌──────────────────────────────────────────────────┐
│  Production Key Management                        │
│                                                   │
│  master_authority  → Squads multisig (3-of-5)    │
│                      Board/C-suite signers        │
│                                                   │
│  pauser            → Hot wallet                   │
│                      24/7 operations team          │
│                      Rapid emergency response      │
│                                                   │
│  minters[]         → Individual hot wallets       │
│                      Tight per-minter quotas       │
│                      Automated mint pipelines      │
│                                                   │
│  burners[]         → Individual hot wallets       │
│                      Redemption processing team    │
│                                                   │
│  blacklister       → Warm wallet                  │
│                      Compliance team               │
│                      Automated sanctions feed      │
│                                                   │
│  seizer            → Cold wallet or multisig      │
│                      Legal team + compliance       │
│                      Used only for court orders    │
└──────────────────────────────────────────────────┘
```

### Key Rotation Schedule

| Role | Rotation Frequency | Notes |
|------|-------------------|-------|
| master_authority | Annually or on personnel change | Two-step transfer prevents lockout |
| pauser | Quarterly | Hot wallet — higher risk |
| minters | Monthly or on compromise | Per-minter quotas limit exposure |
| blacklister | Quarterly | Automate sanctions feeds |
| seizer | Annually | Cold storage, rarely used |

## Audit Trail Format

### On-Chain Event Structure

Every compliance event is emitted via Anchor's `emit!` macro and stored in the transaction log:

```json
{
  "program": "sss_token",
  "event": "AddressBlacklisted",
  "data": {
    "mint": "7Xf3kP9QwR2...",
    "address": "2Rf4jL6mN8...",
    "reason": "OFAC SDN List - Entity: Example Corp",
    "by": "4Gh2mN8qW1..."
  },
  "slot": 234567890,
  "blockTime": 1740412800,
  "signature": "5Kj2txSig..."
}
```

### Indexer Database Schema

The indexer service stores events in SQLite for efficient querying:

```sql
CREATE TABLE compliance_events (
    id INTEGER PRIMARY KEY,
    event_type TEXT NOT NULL,       -- 'blacklist_add', 'blacklist_remove', 'seize', 'freeze', 'thaw'
    mint TEXT NOT NULL,
    target_address TEXT,
    reason TEXT,
    performed_by TEXT NOT NULL,
    amount BIGINT,                  -- for seize events
    slot BIGINT NOT NULL,
    block_time BIGINT NOT NULL,
    signature TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_compliance_mint ON compliance_events(mint);
CREATE INDEX idx_compliance_type ON compliance_events(event_type);
CREATE INDEX idx_compliance_target ON compliance_events(target_address);
CREATE INDEX idx_compliance_time ON compliance_events(block_time);
```

### Audit Report Generation

```bash
# Generate compliance report via API
curl http://localhost:3002/audit-trail \
  -H "Authorization: Bearer <token>" \
  -d '{"from": "2026-01-01", "to": "2026-02-01", "format": "csv"}'

# Or via CLI
sss-token audit-log \
  --after 2026-01-01 \
  --before 2026-02-01 \
  --format csv > report-jan-2026.csv
```

## Sanctions Screening Integration

### OFAC SDN List Integration Points

SSS-2 does not include a built-in sanctions oracle. Instead, it provides integration points for external screening:

```
┌────────────────────────────────────────────────────────┐
│  Sanctions Screening Pipeline                           │
│                                                         │
│  1. External Service                                    │
│     ├── OFAC SDN List API                              │
│     ├── Chainalysis / Elliptic / TRM Labs              │
│     └── Custom sanctions database                       │
│                                                         │
│  2. Compliance Service (services/compliance/)           │
│     ├── Periodic polling of sanctions lists              │
│     ├── Cross-reference with token holder addresses     │
│     └── Auto-generate blacklist commands                │
│                                                         │
│  3. On-Chain Enforcement                                │
│     ├── sss-token blacklist add <address>               │
│     └── Transfer Hook blocks all transfers              │
│                                                         │
│  4. Webhook Notifications                               │
│     ├── Alert compliance team on new matches            │
│     └── Notify legal team for seizure orders            │
└────────────────────────────────────────────────────────┘
```

### Integration Example

```typescript
// Automated sanctions screening (runs periodically)
async function screenHolders(stablecoin: SolanaStablecoin) {
  const holders = await stablecoin.getHolders();

  for (const holder of holders) {
    const sanctioned = await ofacApi.check(holder.address.toBase58());

    if (sanctioned && !(await stablecoin.compliance.isBlacklisted(holder.address))) {
      await stablecoin.compliance.blacklistAdd(
        holder.address,
        `OFAC SDN Match: ${sanctioned.entityName}`,
        blacklisterKeypair
      );
      await notifyComplianceTeam(holder.address, sanctioned);
    }
  }
}
```

## Key Management Recommendations

### Multisig with Squads

For production deployments, use Squads Protocol for multisig control:

```bash
# Create a 3-of-5 multisig for master_authority
# Members: CEO, CFO, CTO, Compliance Officer, Legal Counsel

# Transfer authority to multisig
sss-token transfer-authority <SQUADS_MULTISIG_PDA>
# Accept from multisig (requires 3 of 5 signatures)
```

### Hardware Security Modules (HSMs)

For the seizer role and master_authority:
- Use Solana-compatible HSMs (e.g., Fireblocks, Dfns)
- Configure approval workflows for high-impact operations
- Maintain offline backup of key material

## Compliance Comparison

### SSS-2 vs USDC vs USDT vs PYUSD

| Capability | SSS-2 | USDC (Solana) | USDT (Solana) | PYUSD (Solana) |
|-----------|-------|---------------|---------------|----------------|
| Freeze | FreezeAuthority PDA | FreezeAuthority multisig | FreezeAuthority multisig | FreezeAuthority multisig |
| Seize | PermanentDelegate PDA | Not available* | Not available* | PermanentDelegate |
| Blacklist | Transfer Hook PDAs | Off-chain + frontend | Off-chain + frontend | Transfer Hook |
| Block transfers | On-chain (automatic) | Off-chain (frontend) | Off-chain (frontend) | On-chain (automatic) |
| Audit trail | On-chain events + indexer | Off-chain logs | Off-chain logs | On-chain events |
| Role separation | 6 configurable roles | Multisig signers | Multisig signers | Multisig signers |
| Open source SDK | Yes | Partial | No | Partial |
| Multi-preset | Yes (SSS-1 / SSS-2) | No | No | No |
| Default frozen accts | Configurable | No | No | Yes |

*USDC and USDT on Solana do not use PermanentDelegate. They rely on freeze + off-chain coordination for asset recovery.

### Key Differentiators

1. **On-chain enforcement** — SSS-2's Transfer Hook enforces blacklist at the protocol level. USDC/USDT rely on frontend cooperation for blocking.

2. **Role granularity** — SSS-2 separates blacklister, seizer, pauser, and authority into distinct keys. Most existing stablecoins use a single multisig for all operations.

3. **SDK-first design** — SSS-2 provides a TypeScript SDK and Rust CLI for all compliance operations, reducing integration friction.

4. **Configurable presets** — The same program supports SSS-1 (no compliance overhead) and SSS-2 (full compliance), allowing issuers to choose based on their regulatory requirements.
