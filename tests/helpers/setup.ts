import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  AccountLayout,
} from "@solana/spl-token";

/** PDA seed constants matching on-chain program */
const STABLECOIN_SEED = Buffer.from("stablecoin");
const ROLES_SEED = Buffer.from("roles");
const BLACKLIST_SEED = Buffer.from("blacklist");

/** Program IDs */
export const SSS_TOKEN_PROGRAM_ID = new PublicKey(
  "AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE"
);
export const SSS_TRANSFER_HOOK_PROGRAM_ID = new PublicKey(
  "Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU"
);

/** Airdrop SOL to a given address */
export async function airdrop(
  connection: Connection,
  address: PublicKey,
  amount: number = 10 * LAMPORTS_PER_SOL
): Promise<void> {
  const sig = await connection.requestAirdrop(address, amount);
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature: sig,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  });
}

/** Derive StablecoinConfig PDA */
export function deriveStablecoinPDA(
  programId: PublicKey,
  mint: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [STABLECOIN_SEED, mint.toBuffer()],
    programId
  );
}

/** Derive RoleConfig PDA */
export function deriveRolePDA(
  programId: PublicKey,
  stablecoinConfig: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ROLES_SEED, stablecoinConfig.toBuffer()],
    programId
  );
}

/** Derive BlacklistEntry PDA */
export function deriveBlacklistPDA(
  programId: PublicKey,
  mint: PublicKey,
  address: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [BLACKLIST_SEED, mint.toBuffer(), address.toBuffer()],
    programId
  );
}

/** Create or get an Associated Token Account for Token-2022 */
export async function getOrCreateAta(
  connection: Connection,
  payer: Keypair,
  mint: PublicKey,
  owner: PublicKey
): Promise<PublicKey> {
  const ata = getAssociatedTokenAddressSync(
    mint,
    owner,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  const accountInfo = await connection.getAccountInfo(ata);
  if (!accountInfo) {
    const ix = createAssociatedTokenAccountInstruction(
      payer.publicKey,
      ata,
      owner,
      mint,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );
    const tx = new Transaction().add(ix);
    await sendAndConfirmTransaction(connection, tx, [payer]);
  }

  return ata;
}

/** Get token balance for a Token-2022 account */
export async function getTokenBalance(
  connection: Connection,
  tokenAccount: PublicKey
): Promise<bigint> {
  const account = await getAccount(
    connection,
    tokenAccount,
    undefined,
    TOKEN_2022_PROGRAM_ID
  );
  return account.amount;
}

/** Check if a token account is frozen */
export async function isAccountFrozen(
  connection: Connection,
  tokenAccount: PublicKey
): Promise<boolean> {
  const account = await getAccount(
    connection,
    tokenAccount,
    undefined,
    TOKEN_2022_PROGRAM_ID
  );
  return account.isFrozen;
}

/** Initialize params for SSS-1 preset */
export function sss1Params(overrides?: Partial<any>): any {
  return {
    name: "Test USD",
    symbol: "TUSD",
    uri: "https://example.com/tusd.json",
    decimals: 6,
    enablePermanentDelegate: false,
    enableTransferHook: false,
    defaultAccountFrozen: false,
    ...overrides,
  };
}

/** Initialize params for SSS-2 preset */
export function sss2Params(overrides?: Partial<any>): any {
  return {
    name: "Compliant USD",
    symbol: "CUSD",
    uri: "https://example.com/cusd.json",
    decimals: 6,
    enablePermanentDelegate: true,
    enableTransferHook: true,
    defaultAccountFrozen: false,
    ...overrides,
  };
}

/**
 * Initialize a stablecoin and return all relevant keypairs and addresses.
 */
export async function initializeStablecoin(
  program: Program<any>,
  authority: Keypair,
  params: any,
  transferHookProgram?: PublicKey
): Promise<{
  mint: Keypair;
  stablecoinConfig: PublicKey;
  stablecoinConfigBump: number;
  roleConfig: PublicKey;
  roleConfigBump: number;
}> {
  const mint = Keypair.generate();
  const [stablecoinConfig, stablecoinConfigBump] = deriveStablecoinPDA(
    program.programId,
    mint.publicKey
  );
  const [roleConfig, roleConfigBump] = deriveRolePDA(
    program.programId,
    stablecoinConfig
  );

  const accounts: any = {
    authority: authority.publicKey,
    mint: mint.publicKey,
    stablecoinConfig,
    roleConfig,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  };

  if (params.enableTransferHook && transferHookProgram) {
    accounts.transferHookProgram = transferHookProgram;
  }

  await program.methods
    .initialize(params)
    .accounts(accounts)
    .signers([authority, mint])
    .rpc();

  return {
    mint,
    stablecoinConfig,
    stablecoinConfigBump,
    roleConfig,
    roleConfigBump,
  };
}

/**
 * Add a minter to the stablecoin role config.
 */
export async function addMinter(
  program: Program<any>,
  authority: Keypair,
  stablecoinConfig: PublicKey,
  roleConfig: PublicKey,
  minterAddress: PublicKey,
  quota: anchor.BN
): Promise<void> {
  await program.methods
    .updateMinter({
      add: { address: minterAddress, quota },
    })
    .accounts({
      authority: authority.publicKey,
      stablecoinConfig,
      roleConfig,
    })
    .signers([authority])
    .rpc();
}

/**
 * Add a burner to the stablecoin role config.
 */
export async function addBurner(
  program: Program<any>,
  authority: Keypair,
  stablecoinConfig: PublicKey,
  roleConfig: PublicKey,
  burnerAddress: PublicKey
): Promise<void> {
  await program.methods
    .updateRoles({
      addBurner: { address: burnerAddress },
    })
    .accounts({
      authority: authority.publicKey,
      stablecoinConfig,
      roleConfig,
    })
    .signers([authority])
    .rpc();
}

/**
 * Mint tokens to a recipient.
 */
export async function mintTokens(
  program: Program<any>,
  minter: Keypair,
  stablecoinConfig: PublicKey,
  roleConfig: PublicKey,
  mint: PublicKey,
  recipientTokenAccount: PublicKey,
  amount: anchor.BN
): Promise<string> {
  return program.methods
    .mintTokens(amount)
    .accounts({
      minter: minter.publicKey,
      stablecoinConfig,
      roleConfig,
      mint,
      recipientTokenAccount,
      tokenProgram: TOKEN_2022_PROGRAM_ID,
    })
    .signers([minter])
    .rpc();
}
