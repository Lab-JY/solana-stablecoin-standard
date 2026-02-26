# SDK Reference

## Installation

```bash
# npm
npm install @stbr/sss-token

# yarn
yarn add @stbr/sss-token

# pnpm
pnpm add @stbr/sss-token
```

### Peer Dependencies

```json
{
  "@coral-xyz/anchor": "^0.31.0",
  "@solana/web3.js": "^1.95.0",
  "@solana/spl-token": "^0.4.0"
}
```

## Presets

The SDK provides two opinionated presets that map to Token-2022 extension configurations:

```typescript
import { Presets } from "@stbr/sss-token";

// SSS-1: Minimal stablecoin
Presets.SSS_1
// Extensions: MintAuthority, FreezeAuthority, MetadataPointer, TokenMetadata

// SSS-2: Compliant stablecoin
Presets.SSS_2
// Extensions: All SSS-1 + PermanentDelegate, TransferHook, DefaultAccountState
```

### Custom Configuration

For advanced use cases, pass a custom config instead of a preset:

```typescript
const stablecoin = await SolanaStablecoin.create(connection, {
  name: "Custom Token",
  symbol: "CUST",
  decimals: 6,
  authority: authority.publicKey,
  enablePermanentDelegate: true,
  enableTransferHook: false,  // Custom: delegate but no hook
  defaultAccountFrozen: false,
});
```

## SolanaStablecoin Class

The main entry point for all stablecoin operations.

### Factory Methods

#### `SolanaStablecoin.create(connection, config): Promise<SolanaStablecoin>`

Creates a new stablecoin on-chain and returns a configured instance.

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";
import { Connection, Keypair } from "@solana/web3.js";

const connection = new Connection("http://localhost:8899");
const authority = Keypair.generate();

const stablecoin = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_1,
  name: "Superteam USD",
  symbol: "stUSD",
  decimals: 6,
  authority: authority.publicKey,
});

console.log("Mint:", stablecoin.mint.toBase58());
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `preset` | `Presets` | No | SSS_1 or SSS_2 (sets extension defaults) |
| `name` | `string` | Yes | Token name (max 32 bytes) |
| `symbol` | `string` | Yes | Token symbol (max 10 bytes) |
| `decimals` | `number` | Yes | Token decimals (typically 6) |
| `authority` | `PublicKey` | Yes | Master authority public key |
| `uri` | `string` | No | Metadata URI (max 200 bytes) |
| `enablePermanentDelegate` | `boolean` | No | Enable seize capability |
| `enableTransferHook` | `boolean` | No | Enable blacklist enforcement |
| `defaultAccountFrozen` | `boolean` | No | New accounts start frozen |

#### `SolanaStablecoin.load(connection, mint): Promise<SolanaStablecoin>`

Loads an existing stablecoin by its mint address.

```typescript
const stablecoin = await SolanaStablecoin.load(
  connection,
  new PublicKey("TokenMintAddress...")
);

const config = await stablecoin.getConfig();
console.log("Name:", config.name);
console.log("Paused:", config.paused);
```

### Core Operations

#### `mint(params): Promise<TransactionSignature>`

Mint tokens to a recipient. Caller must be an authorized minter with sufficient quota.

```typescript
import BN from "bn.js";

const sig = await stablecoin.mint({
  recipient: recipientPubkey,
  amount: new BN(1_000_000_000), // 1,000 tokens (6 decimals)
  minter: minterKeypair,
});
```

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `recipient` | `PublicKey` | Token account owner to receive tokens |
| `amount` | `BN` | Amount in base units (respecting decimals) |
| `minter` | `Keypair` | Minter keypair (must be in minters list) |

#### `burn(params): Promise<TransactionSignature>`

Burn tokens from the burner's token account.

```typescript
const sig = await stablecoin.burn({
  amount: new BN(500_000_000), // 500 tokens
  burner: burnerKeypair,
});
```

#### `freezeAccount(address, signer): Promise<TransactionSignature>`

Freeze a specific token account. Caller must be authority or pauser.

```typescript
const sig = await stablecoin.freezeAccount(
  targetAccountOwner,
  pauserKeypair
);
```

#### `thawAccount(address, signer): Promise<TransactionSignature>`

Thaw a frozen token account.

```typescript
const sig = await stablecoin.thawAccount(
  targetAccountOwner,
  pauserKeypair
);
```

#### `pause(signer): Promise<TransactionSignature>`

Pause all operations globally. Emergency circuit breaker.

```typescript
const sig = await stablecoin.pause(pauserKeypair);
```

#### `unpause(signer): Promise<TransactionSignature>`

Resume operations after a pause.

```typescript
const sig = await stablecoin.unpause(pauserKeypair);
```

#### `getTotalSupply(): Promise<BN>`

Returns the current circulating supply (total_minted - total_burned).

```typescript
const supply = await stablecoin.getTotalSupply();
console.log("Supply:", supply.toString());
```

#### `getConfig(): Promise<StablecoinConfig>`

Returns the full on-chain configuration.

```typescript
const config = await stablecoin.getConfig();
console.log("Authority:", config.authority.toBase58());
console.log("Paused:", config.paused);
console.log("Total minted:", config.totalMinted.toString());
console.log("Compliance enabled:", config.isComplianceEnabled());
```

#### `getRoles(): Promise<RoleConfig>`

Returns role assignments and minter quotas.

```typescript
const roles = await stablecoin.getRoles();
console.log("Pauser:", roles.pauser.toBase58());
for (const minter of roles.minters) {
  console.log(`Minter: ${minter.address.toBase58()}`);
  console.log(`  Quota: ${minter.quota.toString()}`);
  console.log(`  Used: ${minter.minted.toString()}`);
  console.log(`  Remaining: ${minter.remainingQuota().toString()}`);
}
```

### Role Management

#### `updateMinter(action, signer): Promise<TransactionSignature>`

Add, remove, or update a minter's quota.

```typescript
// Add a minter
await stablecoin.updateMinter(
  { action: "add", address: minterPubkey, quota: new BN(1_000_000_000_000) },
  authorityKeypair
);

// Update quota
await stablecoin.updateMinter(
  { action: "updateQuota", address: minterPubkey, newQuota: new BN(2_000_000_000_000) },
  authorityKeypair
);

// Remove a minter
await stablecoin.updateMinter(
  { action: "remove", address: minterPubkey },
  authorityKeypair
);
```

#### `updateRoles(params, signer): Promise<TransactionSignature>`

Update role assignments.

```typescript
await stablecoin.updateRoles(
  {
    pauser: newPauserPubkey,
    blacklister: newBlacklisterPubkey, // SSS-2 only
    seizer: newSeizerPubkey,           // SSS-2 only
  },
  authorityKeypair
);
```

#### `transferAuthority(newAuthority, signer): Promise<TransactionSignature>`

Initiate authority transfer (two-step process).

```typescript
// Step 1: Current authority initiates transfer
await stablecoin.transferAuthority(newAuthorityPubkey, currentAuthorityKeypair);

// Step 2: New authority accepts (called from new authority's context)
await stablecoin.acceptAuthority(newAuthorityKeypair);
```

## ComplianceModule Class

Accessed via `stablecoin.compliance`. Available only for SSS-2 stablecoins. Calling on an SSS-1 stablecoin throws `ComplianceNotEnabled`.

```typescript
const compliance = stablecoin.compliance;
// Throws if stablecoin is SSS-1
```

#### `blacklistAdd(address, reason, signer): Promise<TransactionSignature>`

Add an address to the blacklist. Creates a BlacklistEntry PDA on-chain.

```typescript
const sig = await stablecoin.compliance.blacklistAdd(
  suspectAddress,
  "OFAC SDN List - Entity XYZ",
  blacklisterKeypair
);
```

#### `blacklistRemove(address, signer): Promise<TransactionSignature>`

Remove an address from the blacklist. Closes the BlacklistEntry PDA and reclaims rent.

```typescript
const sig = await stablecoin.compliance.blacklistRemove(
  clearedAddress,
  blacklisterKeypair
);
```

#### `seize(from, to, signer): Promise<TransactionSignature>`

Seize all tokens from an address using the permanent delegate authority.

```typescript
const sig = await stablecoin.compliance.seize(
  targetAddress,
  treasuryAddress,
  seizerKeypair
);
```

#### `isBlacklisted(address): Promise<boolean>`

Check if an address is on the blacklist.

```typescript
const blocked = await stablecoin.compliance.isBlacklisted(someAddress);
if (blocked) {
  console.log("Address is blacklisted");
}
```

#### `getBlacklistEntry(address): Promise<BlacklistEntry | null>`

Get full blacklist entry details.

```typescript
const entry = await stablecoin.compliance.getBlacklistEntry(someAddress);
if (entry) {
  console.log("Reason:", entry.reason);
  console.log("Added at:", new Date(entry.addedAt * 1000));
  console.log("Added by:", entry.addedBy.toBase58());
}
```

## PDA Derivation

```typescript
import { deriveStablecoinConfig, deriveRoleConfig, deriveBlacklistEntry } from "@stbr/sss-token";

// StablecoinConfig PDA
const [configPda, configBump] = deriveStablecoinConfig(mintPubkey, programId);

// RoleConfig PDA
const [rolesPda, rolesBump] = deriveRoleConfig(configPda, programId);

// BlacklistEntry PDA
const [blacklistPda, blacklistBump] = deriveBlacklistEntry(
  mintPubkey,
  targetAddress,
  programId
);
```

## Types

```typescript
interface StablecoinConfig {
  authority: PublicKey;
  mint: PublicKey;
  name: string;
  symbol: string;
  uri: string;
  decimals: number;
  paused: boolean;
  totalMinted: BN;
  totalBurned: BN;
  enablePermanentDelegate: boolean;
  enableTransferHook: boolean;
  defaultAccountFrozen: boolean;
  transferHookProgram: PublicKey | null;
  bump: number;
}

interface RoleConfig {
  stablecoin: PublicKey;
  masterAuthority: PublicKey;
  pauser: PublicKey;
  minters: MinterInfo[];
  burners: PublicKey[];
  blacklister: PublicKey;
  seizer: PublicKey;
  bump: number;
}

interface MinterInfo {
  address: PublicKey;
  quota: BN;
  minted: BN;
  remainingQuota(): BN;
}

interface BlacklistEntry {
  stablecoin: PublicKey;
  address: PublicKey;
  reason: string;
  addedAt: number;
  addedBy: PublicKey;
  bump: number;
}

enum Presets {
  SSS_1 = "sss-1",
  SSS_2 = "sss-2",
}

interface MintParams {
  recipient: PublicKey;
  amount: BN;
  minter: Keypair;
}

interface BurnParams {
  amount: BN;
  burner: Keypair;
}
```

## Error Handling

The SDK wraps Anchor errors into typed exceptions:

```typescript
import { SSSError, ErrorCode } from "@stbr/sss-token";

try {
  await stablecoin.mint({ ... });
} catch (err) {
  if (err instanceof SSSError) {
    switch (err.code) {
      case ErrorCode.Paused:
        console.log("Stablecoin is paused");
        break;
      case ErrorCode.MinterQuotaExceeded:
        console.log("Quota exceeded. Remaining:", err.details.remaining);
        break;
      case ErrorCode.ComplianceNotEnabled:
        console.log("SSS-2 feature not available on SSS-1");
        break;
    }
  }
}
```

## Complete Example: SSS-2 Lifecycle

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import BN from "bn.js";

async function main() {
  const connection = new Connection("http://localhost:8899", "confirmed");
  const authority = Keypair.generate();
  const minter = Keypair.generate();
  const blacklister = Keypair.generate();

  // 1. Create SSS-2 stablecoin
  const coin = await SolanaStablecoin.create(connection, {
    preset: Presets.SSS_2,
    name: "Regulated USD",
    symbol: "rUSD",
    decimals: 6,
    authority: authority.publicKey,
  });
  console.log("Created:", coin.mint.toBase58());

  // 2. Configure roles
  await coin.updateMinter(
    { action: "add", address: minter.publicKey, quota: new BN(10_000_000_000) },
    authority
  );
  await coin.updateRoles({ blacklister: blacklister.publicKey }, authority);

  // 3. Mint tokens
  const userA = Keypair.generate();
  await coin.mint({
    recipient: userA.publicKey,
    amount: new BN(1_000_000_000),
    minter,
  });

  // 4. Blacklist an address
  const suspect = Keypair.generate();
  await coin.compliance.blacklistAdd(
    suspect.publicKey,
    "OFAC SDN List",
    blacklister
  );

  // 5. Verify blacklist
  const isBlocked = await coin.compliance.isBlacklisted(suspect.publicKey);
  console.log("Blocked:", isBlocked); // true

  // 6. Check supply
  const supply = await coin.getTotalSupply();
  console.log("Supply:", supply.toString());
}

main().catch(console.error);
```
