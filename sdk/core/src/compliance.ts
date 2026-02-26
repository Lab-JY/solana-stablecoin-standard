import {
  Connection,
  PublicKey,
  Keypair,
  TransactionSignature,
  SystemProgram,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import { Program, BN } from "@coral-xyz/anchor";
import { BlacklistEntry, AuditEntry } from "./types";
import { deriveBlacklistEntry } from "./pda";

/**
 * ComplianceModule handles SSS-2 compliance operations:
 * blacklist management, token seizure, and audit logging.
 *
 * Operations will fail with ComplianceNotEnabled if the stablecoin
 * was initialized as SSS-1 (without compliance features).
 */
export class ComplianceModule {
  constructor(
    private readonly connection: Connection,
    private readonly program: Program,
    private readonly programId: PublicKey,
    private readonly transferHookProgramId: PublicKey,
    private readonly mint: PublicKey,
    private readonly configAddress: PublicKey,
    private readonly configBump: number,
    private readonly roleConfigAddress: PublicKey
  ) {}

  /**
   * Adds an address to the blacklist.
   * Caller must be the blacklister role.
   */
  async blacklistAdd(
    address: PublicKey,
    reason: string,
    blacklister: Keypair
  ): Promise<TransactionSignature> {
    const [blacklistEntry] = deriveBlacklistEntry(
      this.programId,
      this.mint,
      address
    );

    return this.program.methods
      .addToBlacklist(address, reason)
      .accountsStrict({
        blacklister: blacklister.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        blacklistEntry: blacklistEntry,
        systemProgram: SystemProgram.programId,
      })
      .signers([blacklister])
      .rpc();
  }

  /**
   * Removes an address from the blacklist.
   * Caller must be the blacklister role.
   */
  async blacklistRemove(
    address: PublicKey,
    blacklister: Keypair
  ): Promise<TransactionSignature> {
    const [blacklistEntry] = deriveBlacklistEntry(
      this.programId,
      this.mint,
      address
    );

    return this.program.methods
      .removeFromBlacklist(address)
      .accountsStrict({
        blacklister: blacklister.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        blacklistEntry: blacklistEntry,
      })
      .signers([blacklister])
      .rpc();
  }

  /**
   * Seizes tokens from one account and transfers them to another
   * using the permanent delegate authority.
   * Caller must be the seizer role.
   */
  async seize(
    from: PublicKey,
    to: PublicKey,
    amount: BN,
    seizer: Keypair
  ): Promise<TransactionSignature> {
    const fromAta = getAssociatedTokenAddressSync(
      this.mint,
      from,
      false,
      TOKEN_2022_PROGRAM_ID
    );
    const toAta = getAssociatedTokenAddressSync(
      this.mint,
      to,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    return this.program.methods
      .seize(amount)
      .accountsStrict({
        seizer: seizer.publicKey,
        stablecoinConfig: this.configAddress,
        roleConfig: this.roleConfigAddress,
        mint: this.mint,
        fromTokenAccount: fromAta,
        toTokenAccount: toAta,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
      })
      .signers([seizer])
      .rpc();
  }

  /**
   * Checks whether an address is currently blacklisted.
   */
  async isBlacklisted(address: PublicKey): Promise<boolean> {
    const [blacklistEntry] = deriveBlacklistEntry(
      this.programId,
      this.mint,
      address
    );

    try {
      const account = await this.connection.getAccountInfo(blacklistEntry);
      return account !== null && account.data.length > 0;
    } catch {
      return false;
    }
  }

  /**
   * Fetches the BlacklistEntry for a given address if it exists.
   */
  async getBlacklistEntry(address: PublicKey): Promise<BlacklistEntry | null> {
    const [blacklistEntryAddress] = deriveBlacklistEntry(
      this.programId,
      this.mint,
      address
    );

    try {
      const account = await this.program.account.blacklistEntry.fetch(
        blacklistEntryAddress
      );
      return {
        stablecoin: account.stablecoin as PublicKey,
        address: account.address as PublicKey,
        reason: account.reason as string,
        addedAt: account.addedAt as BN,
        addedBy: account.addedBy as PublicKey,
        bump: account.bump as number,
      };
    } catch {
      return null;
    }
  }

  /**
   * Returns the audit log by fetching recent program transaction logs.
   * Parses Anchor events from transaction log messages.
   */
  async getAuditLog(limit: number = 50): Promise<AuditEntry[]> {
    const signatures = await this.connection.getSignaturesForAddress(
      this.configAddress,
      { limit }
    );

    const entries: AuditEntry[] = [];

    for (const sig of signatures) {
      const tx = await this.connection.getTransaction(sig.signature, {
        maxSupportedTransactionVersion: 0,
      });

      if (!tx?.meta?.logMessages) continue;

      for (const log of tx.meta.logMessages) {
        if (log.includes("AddressBlacklisted")) {
          entries.push({
            action: "blacklist_add",
            address: PublicKey.default,
            by: PublicKey.default,
            timestamp: new BN(sig.blockTime ?? 0),
            signature: sig.signature,
            details: log,
          });
        } else if (log.includes("AddressUnblacklisted")) {
          entries.push({
            action: "blacklist_remove",
            address: PublicKey.default,
            by: PublicKey.default,
            timestamp: new BN(sig.blockTime ?? 0),
            signature: sig.signature,
            details: log,
          });
        } else if (log.includes("TokensSeized")) {
          entries.push({
            action: "seize",
            address: PublicKey.default,
            by: PublicKey.default,
            timestamp: new BN(sig.blockTime ?? 0),
            signature: sig.signature,
            details: log,
          });
        }
      }
    }

    return entries;
  }
}
