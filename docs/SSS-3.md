# SSS-3: Private Stablecoin (Proof of Concept)

## Status: Documentation-Only PoC

SSS-3 is a specification for a privacy-preserving stablecoin using Token-2022's Confidential Transfers extension. This document describes the architecture and limitations. **No on-chain implementation is provided** because Confidential Transfers are currently disabled on Solana mainnet and devnet (since June 2025).

## Motivation

Privacy is a legitimate need for many financial use cases:
- Payroll and salary payments
- Business-to-business settlements
- Medical and insurance payments
- Any transaction where public balance visibility creates risk

SSS-3 would allow stablecoin transfers where the **transfer amount is encrypted** using ElGamal encryption while maintaining the ability for authorized auditors to verify compliance.

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                    SSS-3 Mint Account                      │
│                                                            │
│  ┌──────────────────────────────────────────────────────┐ │
│  │  All SSS-1 extensions:                                │ │
│  │  MintAuthority, FreezeAuthority, Metadata            │ │
│  ├──────────────────────────────────────────────────────┤ │
│  │  ConfidentialTransferMint                             │ │
│  │    authority: StablecoinConfig PDA                    │ │
│  │    auto_approve_new_accounts: true                    │ │
│  │    auditor_elgamal_pubkey: <AUDITOR_KEY>             │ │
│  └──────────────────────────────────────────────────────┘ │
└───────────────────────────────────────────────────────────┘
```

### Token Accounts with Confidential Balances

Each token account that opts into confidential transfers maintains:

```
┌───────────────────────────────────────────┐
│     Token Account + Confidential State     │
│                                            │
│  Standard fields:                          │
│    owner, mint, amount (public)            │
│                                            │
│  Confidential Transfer state:              │
│    pending_balance_lo: ElGamalCiphertext   │
│    pending_balance_hi: ElGamalCiphertext   │
│    available_balance: ElGamalCiphertext    │
│    decryptable_available_balance: AeCiph.  │
│    elgamal_pubkey: ElGamalPubkey           │
│    approved: bool                          │
└───────────────────────────────────────────┘
```

### Transfer Flow

```
Sender                              Receiver
  │                                    │
  │ 1. Generate zero-knowledge proof   │
  │    (range proof + equality proof)  │
  │                                    │
  │ 2. Encrypt amount with:           │
  │    - receiver's ElGamal key        │
  │    - auditor's ElGamal key         │
  │                                    │
  │ 3. Submit confidential_transfer   │
  │    with proofs + ciphertexts       │
  │                                    │
  ▼                                    ▼
┌──────────────────────────────────────────┐
│  Token-2022 Confidential Transfer        │
│                                          │
│  Verify ZK proofs:                       │
│  - Range proof (amount >= 0)             │
│  - Sender has sufficient balance         │
│  - Ciphertexts are consistent            │
│                                          │
│  Update encrypted balances               │
│  (amounts never appear in plaintext)     │
└──────────────────────────────────────────┘
```

## Key Design: Auditor Key

The auditor ElGamal key is critical for regulatory compliance in a privacy context:

```
┌──────────────────────────────────────────────────────────┐
│  Every confidential transfer encrypts the amount with:    │
│                                                           │
│  1. Sender's ElGamal key    → sender can decrypt          │
│  2. Receiver's ElGamal key  → receiver can decrypt        │
│  3. Auditor's ElGamal key   → regulator can decrypt       │
│                                                           │
│  This enables:                                            │
│  - Users see their own balances                           │
│  - Public observers see only ciphertexts                  │
│  - Authorized auditors can decrypt all amounts            │
└──────────────────────────────────────────────────────────┘
```

### Auditor Hierarchy

```
SSS-3 Authority
├── Compliance Auditor (global key)
│   └── Can decrypt ALL transfer amounts
│
├── External Auditor (optional, per-period)
│   └── Provided keys for specific audit windows
│
└── Law Enforcement (on court order)
    └── Auditor key shared under legal process
```

## Current Limitations

### Confidential Transfers Disabled (Since June 2025)

Solana disabled Confidential Transfers on mainnet and devnet in June 2025 due to:
- High compute unit requirements per transfer
- ZK proof verification costs exceeding transaction limits
- Validator performance concerns at scale
- Ongoing work to optimize the cryptographic primitives

**Status as of February 2026:** Still disabled. No timeline for re-enablement has been announced.

### Incompatibility with Transfer Hook

**Critical limitation:** Confidential Transfers and Transfer Hook extensions are **mutually exclusive** on Token-2022.

```
┌──────────────────────────────────────────────┐
│  Extension Compatibility                      │
│                                               │
│  SSS-2: TransferHook + PermanentDelegate      │
│         → blacklist enforcement on transfers   │
│                                               │
│  SSS-3: ConfidentialTransfers                 │
│         → encrypted transfer amounts           │
│                                               │
│  SSS-2 + SSS-3: INCOMPATIBLE                 │
│  Cannot have both Transfer Hook and           │
│  Confidential Transfers on the same mint      │
└──────────────────────────────────────────────┘
```

This means:
- SSS-2 (blacklist via transfer hook) and SSS-3 (confidential transfers) **cannot coexist**
- A stablecoin must choose one: compliance enforcement OR transfer privacy
- A potential workaround (allowlist instead of blocklist) is discussed below

### No Permanent Delegate with Confidential Balances

Seizing confidential balances is technically possible via PermanentDelegate but the seized amount is a ciphertext. The seizer would need the auditor key to know the actual amount seized.

## Future Implementation Roadmap

### Phase 1: Specification (Current)

This document. Defines the architecture, extension configuration, and auditor key model.

### Phase 2: Localnet Prototype (When CT re-enabled)

When Confidential Transfers are re-enabled:

1. Implement SSS-3 initialization with `ConfidentialTransferMint` extension
2. Implement `configure_confidential_account` instruction
3. Implement `confidential_mint` — mint to encrypted balance
4. Implement `confidential_burn` — burn from encrypted balance
5. Test on localnet with `solana-test-validator` (if supported)

### Phase 3: Allowlist Model (Alternative to Transfer Hook)

Since Transfer Hook is incompatible, SSS-3 compliance could use an allowlist model:

```
┌────────────────────────────────────────────────┐
│  SSS-3 Allowlist Compliance Model               │
│                                                 │
│  Instead of blocking blacklisted addresses:     │
│  → Only allow approved addresses to transact    │
│                                                 │
│  Implementation:                                │
│  1. DefaultAccountState = Frozen                │
│  2. Compliance team thaws approved accounts     │
│  3. To "blacklist": re-freeze the account       │
│  4. Achieves same effect without Transfer Hook  │
│                                                 │
│  Tradeoff:                                      │
│  - Higher operational overhead (must approve     │
│    every new account)                            │
│  - Cannot block receiving (only sending)        │
│  - Better UX: no hook CPI overhead on transfers │
└────────────────────────────────────────────────┘
```

### Phase 4: Mainnet Deployment

Requires:
1. Confidential Transfers re-enabled on mainnet
2. Compute budget sufficient for ZK proof verification
3. Auditor key management infrastructure
4. Legal opinion on encrypted-balance stablecoins

## SSS-3 Configuration

```rust
// Future InitializeParams for SSS-3
InitializeParams {
    name: "Private USD",
    symbol: "pUSD",
    decimals: 6,
    enable_permanent_delegate: false,    // Optional
    enable_transfer_hook: false,          // MUST be false (incompatible)
    default_account_frozen: true,         // Allowlist model
    enable_confidential_transfers: true,  // SSS-3 flag
    auditor_elgamal_pubkey: Some(auditor_key),
}
```

## Comparison: SSS-1 vs SSS-2 vs SSS-3

| Feature | SSS-1 | SSS-2 | SSS-3 |
|---------|-------|-------|-------|
| Public balances | Yes | Yes | No (encrypted) |
| Transfer Hook | No | Yes | No (incompatible) |
| Permanent Delegate | No | Yes | Optional |
| Blacklist | No | On-chain | Via freeze (allowlist) |
| Auditor visibility | N/A | Full (public) | Via auditor key |
| GENIUS Act | No | Yes | Partial (auditor key) |
| Status | Production | Production | PoC (blocked) |
| Transfer cost | ~5K CU | ~10K CU | ~200K+ CU |

## Security Considerations

### Auditor Key Compromise

If the auditor ElGamal key is compromised, all historical and future transfer amounts for that mint can be decrypted. Mitigations:
- Rotate auditor key periodically
- Use hardware security modules (HSMs) for key storage
- Implement key ceremony with multi-party computation

### Encrypted Balance Seizure

Seizing from encrypted balances requires careful handling:
- The program can transfer ciphertexts via PermanentDelegate
- The treasury must configure a ConfidentialTransfer account
- The auditor key is needed to determine the actual amount seized
- Legal proceedings may require plaintext amount disclosure (auditor decrypts)

### ZK Proof Costs

Confidential transfers require zero-knowledge proofs that are expensive to verify:
- Current estimates: 200K-400K compute units per transfer
- Solana transaction limit: 1.4M compute units (with priority)
- Leaves limited room for other operations in the same transaction
- This is the primary reason CT is currently disabled
