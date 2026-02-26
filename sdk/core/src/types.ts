import { PublicKey } from "@solana/web3.js";
import BN from "bn.js";

/** Matches on-chain StablecoinConfig account */
export interface StablecoinConfig {
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

/** Matches on-chain RoleConfig account */
export interface RoleConfig {
  stablecoin: PublicKey;
  masterAuthority: PublicKey;
  pauser: PublicKey;
  minters: MinterInfo[];
  burners: PublicKey[];
  blacklister: PublicKey;
  seizer: PublicKey;
  bump: number;
}

/** Matches on-chain MinterInfo struct */
export interface MinterInfo {
  address: PublicKey;
  quota: BN;
  minted: BN;
}

/** Matches on-chain BlacklistEntry account */
export interface BlacklistEntry {
  stablecoin: PublicKey;
  address: PublicKey;
  reason: string;
  addedAt: BN;
  addedBy: PublicKey;
  bump: number;
}

/** Parameters for the initialize instruction */
export interface InitializeParams {
  name: string;
  symbol: string;
  uri: string;
  decimals: number;
  enablePermanentDelegate: boolean;
  enableTransferHook: boolean;
  defaultAccountFrozen: boolean;
}

/** Parameters for the mint instruction */
export interface MintParams {
  recipient: PublicKey;
  amount: BN;
}

/** Parameters for the burn instruction */
export interface BurnParams {
  amount: BN;
  tokenAccount: PublicKey;
}

/** Parameters for creating a stablecoin (used by the SDK factory) */
export interface CreateConfig {
  connection: any;
  payer: any;
  params: InitializeParams;
  transferHookProgramId?: PublicKey;
}

/** Holder info with balance */
export interface HolderInfo {
  address: PublicKey;
  tokenAccount: PublicKey;
  balance: BN;
  isFrozen: boolean;
}

/** Audit trail entry for compliance operations */
export interface AuditEntry {
  action: string;
  address: PublicKey;
  by: PublicKey;
  timestamp: BN;
  signature: string;
  details?: string;
}

/** Minter update actions */
export enum MinterAction {
  Add = "add",
  Remove = "remove",
  UpdateQuota = "update_quota",
}

/** Update minter parameters */
export interface UpdateMinterParams {
  action: MinterAction;
  address: PublicKey;
  quota?: BN;
}

/** Role types for update_roles */
export enum RoleType {
  Pauser = "pauser",
  Blacklister = "blacklister",
  Seizer = "seizer",
}

/** Update roles parameters */
export interface UpdateRolesParams {
  role: RoleType;
  address: PublicKey;
}
