import { Connection, PublicKey } from "@solana/web3.js";
import { AnchorProvider, Program } from "@coral-xyz/anchor";

/** SSS Token program ID */
export const SSS_TOKEN_PROGRAM_ID = new PublicKey(
  "AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE"
);

/** SSS Transfer Hook program ID */
export const SSS_TRANSFER_HOOK_PROGRAM_ID = new PublicKey(
  "Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU"
);

/** SSS Oracle program ID */
export const SSS_ORACLE_PROGRAM_ID = new PublicKey(
  "8kRVqx5JN2rSfn2haXBqgaLnQBrXzNSYj6PH9fKRk5bN"
);

/** PDA derivation seeds */
const STABLECOIN_SEED = Buffer.from("stablecoin");
const ROLES_SEED = Buffer.from("roles");
const BLACKLIST_SEED = Buffer.from("blacklist");

/** Derive StablecoinConfig PDA */
export function deriveStablecoinConfig(
  mint: PublicKey,
  programId: PublicKey = SSS_TOKEN_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [STABLECOIN_SEED, mint.toBuffer()],
    programId
  );
}

/** Derive RoleConfig PDA */
export function deriveRoleConfig(
  stablecoinConfig: PublicKey,
  programId: PublicKey = SSS_TOKEN_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ROLES_SEED, stablecoinConfig.toBuffer()],
    programId
  );
}

/** Derive BlacklistEntry PDA */
export function deriveBlacklistEntry(
  mint: PublicKey,
  address: PublicKey,
  programId: PublicKey = SSS_TOKEN_PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [BLACKLIST_SEED, mint.toBuffer(), address.toBuffer()],
    programId
  );
}

/** Get an Anchor provider from the connected wallet */
export function getProvider(
  connection: Connection,
  wallet: any
): AnchorProvider {
  return new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });
}

/**
 * Fetch and parse StablecoinConfig account data.
 * Returns null if the account doesn't exist.
 */
export async function fetchStablecoinConfig(
  connection: Connection,
  mint: PublicKey
): Promise<{
  authority: PublicKey;
  name: string;
  symbol: string;
  decimals: number;
  paused: boolean;
  totalMinted: bigint;
  totalBurned: bigint;
  enablePermanentDelegate: boolean;
  enableTransferHook: boolean;
  defaultAccountFrozen: boolean;
} | null> {
  const [configPda] = deriveStablecoinConfig(mint);
  const accountInfo = await connection.getAccountInfo(configPda);
  if (!accountInfo) return null;

  const data = accountInfo.data;
  if (data.length < 8) return null;

  // Skip 8-byte discriminator
  let offset = 8;

  const authority = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  // Skip mint (32 bytes)
  offset += 32;

  // Parse strings
  const nameLen = data.readUInt32LE(offset);
  offset += 4;
  const name = data.subarray(offset, offset + nameLen).toString("utf8");
  offset += nameLen;

  const symbolLen = data.readUInt32LE(offset);
  offset += 4;
  const symbol = data.subarray(offset, offset + symbolLen).toString("utf8");
  offset += symbolLen;

  const uriLen = data.readUInt32LE(offset);
  offset += 4;
  offset += uriLen; // skip uri

  const decimals = data[offset];
  offset += 1;

  const paused = data[offset] !== 0;
  offset += 1;

  const totalMinted = data.readBigUInt64LE(offset);
  offset += 8;

  const totalBurned = data.readBigUInt64LE(offset);
  offset += 8;

  const enablePermanentDelegate = data[offset] !== 0;
  offset += 1;

  const enableTransferHook = data[offset] !== 0;
  offset += 1;

  const defaultAccountFrozen = data[offset] !== 0;

  return {
    authority,
    name,
    symbol,
    decimals,
    paused,
    totalMinted,
    totalBurned,
    enablePermanentDelegate,
    enableTransferHook,
    defaultAccountFrozen,
  };
}
