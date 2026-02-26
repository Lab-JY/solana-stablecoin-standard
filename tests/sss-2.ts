import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { expect } from "chai";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createTransferCheckedInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  airdrop,
  deriveStablecoinPDA,
  deriveRolePDA,
  deriveBlacklistPDA,
  initializeStablecoin,
  getOrCreateAta,
  getTokenBalance,
  sss2Params,
  addMinter,
  mintTokens,
  SSS_TOKEN_PROGRAM_ID,
  SSS_TRANSFER_HOOK_PROGRAM_ID,
} from "./helpers/setup";

describe("SSS-2 Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<any>;
  const hookProgram = anchor.workspace.SssTransferHook as Program<any>;

  const authority = Keypair.generate();
  const minter = Keypair.generate();
  const userA = Keypair.generate();
  const userB = Keypair.generate();
  const treasury = Keypair.generate();

  let mint: Keypair;
  let stablecoinConfig: PublicKey;
  let stablecoinConfigBump: number;
  let roleConfig: PublicKey;
  let roleConfigBump: number;

  const MINT_AMOUNT = new anchor.BN(1_000_000_000); // 1000 tokens
  const MINTER_QUOTA = new anchor.BN(100_000_000_000); // 100,000 tokens

  before(async () => {
    await Promise.all([
      airdrop(provider.connection, authority.publicKey),
      airdrop(provider.connection, minter.publicKey),
      airdrop(provider.connection, userA.publicKey),
      airdrop(provider.connection, userB.publicKey),
      airdrop(provider.connection, treasury.publicKey),
    ]);
  });

  describe("1. Initialize SSS-2 stablecoin", () => {
    it("should initialize an SSS-2 stablecoin with compliance extensions", async () => {
      const result = await initializeStablecoin(
        program,
        authority,
        sss2Params(),
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );
      mint = result.mint;
      stablecoinConfig = result.stablecoinConfig;
      stablecoinConfigBump = result.stablecoinConfigBump;
      roleConfig = result.roleConfig;
      roleConfigBump = result.roleConfigBump;

      // Verify config state
      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.authority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(config.name).to.equal("Compliant USD");
      expect(config.symbol).to.equal("CUSD");
      expect(config.decimals).to.equal(6);
      expect(config.enablePermanentDelegate).to.be.true;
      expect(config.enableTransferHook).to.be.true;
      expect(config.defaultAccountFrozen).to.be.false;
      expect(config.transferHookProgram).to.not.be.null;
      expect(config.transferHookProgram.toString()).to.equal(
        SSS_TRANSFER_HOOK_PROGRAM_ID.toString()
      );
    });

    it("should verify role config has compliance roles", async () => {
      const roles = await program.account.roleConfig.fetch(roleConfig);
      expect(roles.masterAuthority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(roles.blacklister.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(roles.seizer.toString()).to.equal(
        authority.publicKey.toString()
      );
    });
  });

  describe("2. Initialize transfer hook extra account metas", () => {
    it("should initialize the extra account meta list for the transfer hook", async () => {
      // The ExtraAccountMetas PDA is derived from the mint and the transfer hook program
      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );

      await hookProgram.methods
        .initializeExtraAccountMetaList()
        .accounts({
          payer: authority.publicKey,
          mint: mint.publicKey,
          extraAccountMetaList,
          sssTokenProgram: SSS_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      // Verify the account exists
      const accountInfo = await provider.connection.getAccountInfo(
        extraAccountMetaList
      );
      expect(accountInfo).to.not.be.null;
    });
  });

  describe("3. Mint tokens and setup", () => {
    it("should add minter and mint tokens to users", async () => {
      await addMinter(
        program,
        authority,
        stablecoinConfig,
        roleConfig,
        minter.publicKey,
        MINTER_QUOTA
      );

      // Create ATAs for all users
      const userAAta = await getOrCreateAta(
        provider.connection,
        authority,
        mint.publicKey,
        userA.publicKey
      );
      const userBAta = await getOrCreateAta(
        provider.connection,
        authority,
        mint.publicKey,
        userB.publicKey
      );
      await getOrCreateAta(
        provider.connection,
        authority,
        mint.publicKey,
        treasury.publicKey
      );

      // Mint to both users
      await mintTokens(
        program,
        minter,
        stablecoinConfig,
        roleConfig,
        mint.publicKey,
        userAAta,
        MINT_AMOUNT
      );
      await mintTokens(
        program,
        minter,
        stablecoinConfig,
        roleConfig,
        mint.publicKey,
        userBAta,
        MINT_AMOUNT
      );

      const balanceA = await getTokenBalance(provider.connection, userAAta);
      const balanceB = await getTokenBalance(provider.connection, userBAta);
      expect(balanceA.toString()).to.equal(MINT_AMOUNT.toString());
      expect(balanceB.toString()).to.equal(MINT_AMOUNT.toString());
    });

    it("should allow normal transfer between non-blacklisted users", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const userBAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const transferAmount = 100_000_000; // 100 tokens

      // For Token-2022 with transfer hook, we need to include the extra account metas
      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );

      // Derive blacklist PDAs for sender and receiver
      const [senderBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );
      const [receiverBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userB.publicKey
      );

      const ix = createTransferCheckedInstruction(
        userAAta,
        mint.publicKey,
        userBAta,
        userA.publicKey,
        transferAmount,
        6,
        [],
        TOKEN_2022_PROGRAM_ID
      );

      // Append the extra accounts required by the transfer hook
      ix.keys.push(
        { pubkey: extraAccountMetaList, isSigner: false, isWritable: false },
        { pubkey: senderBlacklist, isSigner: false, isWritable: false },
        { pubkey: receiverBlacklist, isSigner: false, isWritable: false },
        {
          pubkey: SSS_TRANSFER_HOOK_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        }
      );

      const tx = new Transaction().add(ix);
      await sendAndConfirmTransaction(provider.connection, tx, [userA]);

      const balanceA = await getTokenBalance(provider.connection, userAAta);
      const balanceB = await getTokenBalance(provider.connection, userBAta);
      expect(balanceA.toString()).to.equal("900000000"); // 900 tokens
      expect(balanceB.toString()).to.equal("1100000000"); // 1100 tokens
    });
  });

  describe("4. Blacklist address", () => {
    it("should blacklist user A", async () => {
      const [blacklistEntry] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );

      await program.methods
        .addToBlacklist(userA.publicKey, "Sanctions compliance")
        .accounts({
          blacklister: authority.publicKey,
          stablecoinConfig,
          roleConfig,
          blacklistEntry,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      // Verify blacklist entry
      const entry = await program.account.blacklistEntry.fetch(blacklistEntry);
      expect(entry.address.toString()).to.equal(userA.publicKey.toString());
      expect(entry.reason).to.equal("Sanctions compliance");
      expect(entry.addedBy.toString()).to.equal(
        authority.publicKey.toString()
      );
    });

    it("should fail to transfer from blacklisted sender", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const userBAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );
      const [senderBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );
      const [receiverBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userB.publicKey
      );

      const ix = createTransferCheckedInstruction(
        userAAta,
        mint.publicKey,
        userBAta,
        userA.publicKey,
        50_000_000, // 50 tokens
        6,
        [],
        TOKEN_2022_PROGRAM_ID
      );

      ix.keys.push(
        { pubkey: extraAccountMetaList, isSigner: false, isWritable: false },
        { pubkey: senderBlacklist, isSigner: false, isWritable: false },
        { pubkey: receiverBlacklist, isSigner: false, isWritable: false },
        {
          pubkey: SSS_TRANSFER_HOOK_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        }
      );

      const tx = new Transaction().add(ix);
      try {
        await sendAndConfirmTransaction(provider.connection, tx, [userA]);
        expect.fail("Transfer from blacklisted sender should have failed");
      } catch (err: any) {
        // Transfer hook rejects the transfer
        expect(err.toString()).to.include("custom program error");
      }
    });

    it("should fail to transfer to blacklisted receiver", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const userBAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );
      const [senderBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userB.publicKey
      );
      const [receiverBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );

      const ix = createTransferCheckedInstruction(
        userBAta,
        mint.publicKey,
        userAAta,
        userB.publicKey,
        50_000_000,
        6,
        [],
        TOKEN_2022_PROGRAM_ID
      );

      ix.keys.push(
        { pubkey: extraAccountMetaList, isSigner: false, isWritable: false },
        { pubkey: senderBlacklist, isSigner: false, isWritable: false },
        { pubkey: receiverBlacklist, isSigner: false, isWritable: false },
        {
          pubkey: SSS_TRANSFER_HOOK_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        }
      );

      const tx = new Transaction().add(ix);
      try {
        await sendAndConfirmTransaction(provider.connection, tx, [userB]);
        expect.fail("Transfer to blacklisted receiver should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("custom program error");
      }
    });
  });

  describe("5. Remove from blacklist", () => {
    it("should remove user A from blacklist", async () => {
      const [blacklistEntry] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );

      await program.methods
        .removeFromBlacklist(userA.publicKey)
        .accounts({
          blacklister: authority.publicKey,
          stablecoinConfig,
          roleConfig,
          blacklistEntry,
        })
        .signers([authority])
        .rpc();

      // Verify entry is gone
      const accountInfo = await provider.connection.getAccountInfo(
        blacklistEntry
      );
      expect(accountInfo).to.be.null;
    });

    it("should allow transfer after removal from blacklist", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const userBAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );
      const [senderBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userA.publicKey
      );
      const [receiverBlacklist] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        mint.publicKey,
        userB.publicKey
      );

      const transferAmount = 50_000_000;

      const ix = createTransferCheckedInstruction(
        userAAta,
        mint.publicKey,
        userBAta,
        userA.publicKey,
        transferAmount,
        6,
        [],
        TOKEN_2022_PROGRAM_ID
      );

      ix.keys.push(
        { pubkey: extraAccountMetaList, isSigner: false, isWritable: false },
        { pubkey: senderBlacklist, isSigner: false, isWritable: false },
        { pubkey: receiverBlacklist, isSigner: false, isWritable: false },
        {
          pubkey: SSS_TRANSFER_HOOK_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        }
      );

      const tx = new Transaction().add(ix);
      await sendAndConfirmTransaction(provider.connection, tx, [userA]);

      const balanceA = await getTokenBalance(provider.connection, userAAta);
      expect(balanceA.toString()).to.equal("850000000"); // 900 - 50 = 850 tokens
    });
  });

  describe("6. Seize tokens via permanent delegate", () => {
    it("should seize tokens from user B to treasury", async () => {
      const userBAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userB.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );
      const treasuryAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        treasury.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const balanceBefore = await getTokenBalance(
        provider.connection,
        userBAta
      );
      const seizeAmount = new anchor.BN(500_000_000); // 500 tokens

      await program.methods
        .seize(seizeAmount)
        .accounts({
          seizer: authority.publicKey,
          stablecoinConfig,
          roleConfig,
          mint: mint.publicKey,
          fromTokenAccount: userBAta,
          toTokenAccount: treasuryAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      const balanceB = await getTokenBalance(provider.connection, userBAta);
      const balanceTreasury = await getTokenBalance(
        provider.connection,
        treasuryAta
      );

      // Verify tokens were seized without user B's signature
      const expectedBalance = BigInt(balanceBefore.toString()) - BigInt(500_000_000);
      expect(balanceB.toString()).to.equal(expectedBalance.toString());
      expect(balanceTreasury.toString()).to.equal("500000000");
    });

    it("should have used permanent delegate authority (no user B signature needed)", async () => {
      // The fact that the seize succeeded without user B as a signer
      // proves the permanent delegate was used
      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.enablePermanentDelegate).to.be.true;
    });
  });

  describe("7. Full SSS-2 lifecycle", () => {
    let lifecycleMint: Keypair;
    let lifecycleConfig: PublicKey;
    let lifecycleRoleConfig: PublicKey;
    const lifecycleUser = Keypair.generate();

    before(async () => {
      await airdrop(provider.connection, lifecycleUser.publicKey);
    });

    it("should complete full lifecycle: init -> mint -> transfer -> blacklist -> seize -> unblacklist", async () => {
      // Step 1: Initialize
      const result = await initializeStablecoin(
        program,
        authority,
        sss2Params({
          name: "Lifecycle USD",
          symbol: "LUSD",
        }),
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );
      lifecycleMint = result.mint;
      lifecycleConfig = result.stablecoinConfig;
      lifecycleRoleConfig = result.roleConfig;

      // Step 2: Setup transfer hook
      const [extraAccountMetaList] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("extra-account-metas"),
          lifecycleMint.publicKey.toBuffer(),
        ],
        SSS_TRANSFER_HOOK_PROGRAM_ID
      );

      await hookProgram.methods
        .initializeExtraAccountMetaList()
        .accounts({
          payer: authority.publicKey,
          mint: lifecycleMint.publicKey,
          extraAccountMetaList,
          sssTokenProgram: SSS_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      // Step 3: Add minter and mint
      await addMinter(
        program,
        authority,
        lifecycleConfig,
        lifecycleRoleConfig,
        minter.publicKey,
        new anchor.BN(100_000_000_000)
      );

      const lifecycleUserAta = await getOrCreateAta(
        provider.connection,
        authority,
        lifecycleMint.publicKey,
        lifecycleUser.publicKey
      );
      const treasuryAta = await getOrCreateAta(
        provider.connection,
        authority,
        lifecycleMint.publicKey,
        treasury.publicKey
      );

      await mintTokens(
        program,
        minter,
        lifecycleConfig,
        lifecycleRoleConfig,
        lifecycleMint.publicKey,
        lifecycleUserAta,
        new anchor.BN(5_000_000_000) // 5000 tokens
      );

      let balance = await getTokenBalance(
        provider.connection,
        lifecycleUserAta
      );
      expect(balance.toString()).to.equal("5000000000");

      // Step 4: Blacklist the user
      const [blacklistEntry] = deriveBlacklistPDA(
        SSS_TOKEN_PROGRAM_ID,
        lifecycleMint.publicKey,
        lifecycleUser.publicKey
      );

      await program.methods
        .addToBlacklist(lifecycleUser.publicKey, "Suspicious activity")
        .accounts({
          blacklister: authority.publicKey,
          stablecoinConfig: lifecycleConfig,
          roleConfig: lifecycleRoleConfig,
          blacklistEntry,
          systemProgram: SystemProgram.programId,
        })
        .signers([authority])
        .rpc();

      // Verify blacklisted
      const entry = await program.account.blacklistEntry.fetch(blacklistEntry);
      expect(entry.address.toString()).to.equal(
        lifecycleUser.publicKey.toString()
      );

      // Step 5: Seize tokens
      await program.methods
        .seize(new anchor.BN(5_000_000_000))
        .accounts({
          seizer: authority.publicKey,
          stablecoinConfig: lifecycleConfig,
          roleConfig: lifecycleRoleConfig,
          mint: lifecycleMint.publicKey,
          fromTokenAccount: lifecycleUserAta,
          toTokenAccount: treasuryAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      balance = await getTokenBalance(provider.connection, lifecycleUserAta);
      expect(balance.toString()).to.equal("0");

      const treasuryBalance = await getTokenBalance(
        provider.connection,
        treasuryAta
      );
      expect(treasuryBalance.toString()).to.equal("5000000000");

      // Step 6: Unblacklist
      await program.methods
        .removeFromBlacklist(lifecycleUser.publicKey)
        .accounts({
          blacklister: authority.publicKey,
          stablecoinConfig: lifecycleConfig,
          roleConfig: lifecycleRoleConfig,
          blacklistEntry,
        })
        .signers([authority])
        .rpc();

      // Verify unblacklisted
      const accountInfo = await provider.connection.getAccountInfo(
        blacklistEntry
      );
      expect(accountInfo).to.be.null;

      // Verify supply is correct
      const config = await program.account.stablecoinConfig.fetch(
        lifecycleConfig
      );
      expect(config.totalMinted.toString()).to.equal("5000000000");
    });
  });
});
