import { PublicKey } from "@solana/web3.js";

/** PDA seed constants matching on-chain program */
const STABLECOIN_SEED = Buffer.from("stablecoin");
const ROLES_SEED = Buffer.from("roles");
const BLACKLIST_SEED = Buffer.from("blacklist");

/**
 * Derives the StablecoinConfig PDA address.
 * Seeds: ["stablecoin", mint]
 */
export function deriveStablecoinConfig(
  programId: PublicKey,
  mint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [STABLECOIN_SEED, mint.toBuffer()],
    programId
  );
}

/**
 * Derives the RoleConfig PDA address.
 * Seeds: ["roles", stablecoinConfig]
 */
export function deriveRoleConfig(
  programId: PublicKey,
  stablecoinConfig: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ROLES_SEED, stablecoinConfig.toBuffer()],
    programId
  );
}

/**
 * Derives the BlacklistEntry PDA address.
 * Seeds: ["blacklist", mint, address]
 */
export function deriveBlacklistEntry(
  programId: PublicKey,
  mint: PublicKey,
  address: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [BLACKLIST_SEED, mint.toBuffer(), address.toBuffer()],
    programId
  );
}
