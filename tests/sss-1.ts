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
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createTransferCheckedInstruction,
} from "@solana/spl-token";
import {
  airdrop,
  deriveStablecoinPDA,
  deriveRolePDA,
  initializeStablecoin,
  getOrCreateAta,
  getTokenBalance,
  isAccountFrozen,
  sss1Params,
  addMinter,
  mintTokens,
  SSS_TOKEN_PROGRAM_ID,
} from "./helpers/setup";

describe("SSS-1 Integration Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SssToken as Program<any>;
  const authority = Keypair.generate();
  const minter = Keypair.generate();
  const burner = Keypair.generate();
  const userA = Keypair.generate();
  const userB = Keypair.generate();
  const unauthorized = Keypair.generate();

  let mint: Keypair;
  let stablecoinConfig: PublicKey;
  let stablecoinConfigBump: number;
  let roleConfig: PublicKey;
  let roleConfigBump: number;

  const MINT_AMOUNT = new anchor.BN(1_000_000_000); // 1000 tokens (6 decimals)
  const MINTER_QUOTA = new anchor.BN(10_000_000_000); // 10000 tokens

  before(async () => {
    // Fund all test accounts
    await Promise.all([
      airdrop(provider.connection, authority.publicKey),
      airdrop(provider.connection, minter.publicKey),
      airdrop(provider.connection, burner.publicKey),
      airdrop(provider.connection, userA.publicKey),
      airdrop(provider.connection, userB.publicKey),
      airdrop(provider.connection, unauthorized.publicKey),
    ]);
  });

  describe("1. Initialize SSS-1 stablecoin", () => {
    it("should initialize an SSS-1 stablecoin without compliance extensions", async () => {
      const result = await initializeStablecoin(
        program,
        authority,
        sss1Params()
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
      expect(config.mint.toString()).to.equal(mint.publicKey.toString());
      expect(config.name).to.equal("Test USD");
      expect(config.symbol).to.equal("TUSD");
      expect(config.decimals).to.equal(6);
      expect(config.paused).to.be.false;
      expect(config.enablePermanentDelegate).to.be.false;
      expect(config.enableTransferHook).to.be.false;
      expect(config.defaultAccountFrozen).to.be.false;
      expect(config.totalMinted.toNumber()).to.equal(0);
      expect(config.totalBurned.toNumber()).to.equal(0);
    });

    it("should initialize role config with authority as all roles", async () => {
      const roles = await program.account.roleConfig.fetch(roleConfig);
      expect(roles.masterAuthority.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(roles.pauser.toString()).to.equal(
        authority.publicKey.toString()
      );
      expect(roles.minters).to.have.length(0);
      expect(roles.burners).to.have.length(0);
    });
  });

  describe("2. Add minter with quota", () => {
    it("should add a minter with the specified quota", async () => {
      await addMinter(
        program,
        authority,
        stablecoinConfig,
        roleConfig,
        minter.publicKey,
        MINTER_QUOTA
      );

      const roles = await program.account.roleConfig.fetch(roleConfig);
      expect(roles.minters).to.have.length(1);
      expect(roles.minters[0].address.toString()).to.equal(
        minter.publicKey.toString()
      );
      expect(roles.minters[0].quota.toString()).to.equal(
        MINTER_QUOTA.toString()
      );
      expect(roles.minters[0].minted.toNumber()).to.equal(0);
    });
  });

  describe("3. Mint tokens to recipient", () => {
    it("should mint tokens to user A", async () => {
      const userAAta = await getOrCreateAta(
        provider.connection,
        authority,
        mint.publicKey,
        userA.publicKey
      );

      await mintTokens(
        program,
        minter,
        stablecoinConfig,
        roleConfig,
        mint.publicKey,
        userAAta,
        MINT_AMOUNT
      );

      const balance = await getTokenBalance(provider.connection, userAAta);
      expect(balance.toString()).to.equal(MINT_AMOUNT.toString());

      // Verify config total_minted updated
      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.totalMinted.toString()).to.equal(MINT_AMOUNT.toString());
    });

    it("should mint tokens to user B", async () => {
      const userBAta = await getOrCreateAta(
        provider.connection,
        authority,
        mint.publicKey,
        userB.publicKey
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

      const balance = await getTokenBalance(provider.connection, userBAta);
      expect(balance.toString()).to.equal(MINT_AMOUNT.toString());
    });
  });

  describe("4. Transfer tokens", () => {
    it("should transfer tokens from user A to user B using transfer_checked", async () => {
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
      const ix = createTransferCheckedInstruction(
        userAAta,
        mint.publicKey,
        userBAta,
        userA.publicKey,
        transferAmount,
        6, // decimals
        [],
        TOKEN_2022_PROGRAM_ID
      );

      const tx = new Transaction().add(ix);
      await sendAndConfirmTransaction(provider.connection, tx, [userA]);

      const balanceA = await getTokenBalance(provider.connection, userAAta);
      const balanceB = await getTokenBalance(provider.connection, userBAta);

      expect(balanceA.toString()).to.equal("900000000"); // 900 tokens
      expect(balanceB.toString()).to.equal("1100000000"); // 1100 tokens
    });
  });

  describe("5. Freeze account", () => {
    it("should freeze user A's token account", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      await program.methods
        .freezeAccount()
        .accounts({
          authority: authority.publicKey,
          stablecoinConfig,
          roleConfig,
          mint: mint.publicKey,
          tokenAccount: userAAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      const frozen = await isAccountFrozen(provider.connection, userAAta);
      expect(frozen).to.be.true;
    });

    it("should fail to transfer from a frozen account", async () => {
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

      const ix = createTransferCheckedInstruction(
        userAAta,
        mint.publicKey,
        userBAta,
        userA.publicKey,
        100_000_000,
        6,
        [],
        TOKEN_2022_PROGRAM_ID
      );

      const tx = new Transaction().add(ix);
      try {
        await sendAndConfirmTransaction(provider.connection, tx, [userA]);
        expect.fail("Transfer from frozen account should have failed");
      } catch (err: any) {
        // Token-2022 returns AccountFrozen error
        expect(err.toString()).to.include("0x11"); // AccountFrozen
      }
    });
  });

  describe("6. Thaw account", () => {
    it("should thaw user A's token account", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      await program.methods
        .thawAccount()
        .accounts({
          authority: authority.publicKey,
          stablecoinConfig,
          roleConfig,
          mint: mint.publicKey,
          tokenAccount: userAAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([authority])
        .rpc();

      const frozen = await isAccountFrozen(provider.connection, userAAta);
      expect(frozen).to.be.false;
    });

    it("should allow transfer after thawing", async () => {
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

      const transferAmount = 50_000_000; // 50 tokens
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

      const tx = new Transaction().add(ix);
      await sendAndConfirmTransaction(provider.connection, tx, [userA]);

      const balanceA = await getTokenBalance(provider.connection, userAAta);
      expect(balanceA.toString()).to.equal("850000000"); // 850 tokens
    });
  });

  describe("7. Pause and unpause", () => {
    it("should pause the stablecoin", async () => {
      await program.methods
        .pause()
        .accounts({
          pauser: authority.publicKey,
          stablecoinConfig,
          roleConfig,
        })
        .signers([authority])
        .rpc();

      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.paused).to.be.true;
    });

    it("should fail to mint while paused", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      try {
        await mintTokens(
          program,
          minter,
          stablecoinConfig,
          roleConfig,
          mint.publicKey,
          userAAta,
          MINT_AMOUNT
        );
        expect.fail("Mint while paused should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("Paused");
      }
    });

    it("should unpause the stablecoin", async () => {
      await program.methods
        .unpause()
        .accounts({
          pauser: authority.publicKey,
          stablecoinConfig,
          roleConfig,
        })
        .signers([authority])
        .rpc();

      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.paused).to.be.false;
    });

    it("should allow mint after unpausing", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      const mintAmount = new anchor.BN(100_000_000); // 100 tokens
      await mintTokens(
        program,
        minter,
        stablecoinConfig,
        roleConfig,
        mint.publicKey,
        userAAta,
        mintAmount
      );

      const balance = await getTokenBalance(provider.connection, userAAta);
      // 850 + 100 = 950 tokens
      expect(balance.toString()).to.equal("950000000");
    });
  });

  describe("8. Burn tokens", () => {
    it("should burn tokens from user A's account", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      // First add user A as a burner (or use authority)
      // For simplicity, we burn using userA who owns the tokens
      const burnAmount = new anchor.BN(200_000_000); // 200 tokens

      await program.methods
        .burnTokens(burnAmount)
        .accounts({
          burner: userA.publicKey,
          stablecoinConfig,
          roleConfig,
          mint: mint.publicKey,
          burnerTokenAccount: userAAta,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([userA])
        .rpc();

      const balance = await getTokenBalance(provider.connection, userAAta);
      expect(balance.toString()).to.equal("750000000"); // 950 - 200 = 750 tokens

      // Verify total_burned updated
      const config = await program.account.stablecoinConfig.fetch(
        stablecoinConfig
      );
      expect(config.totalBurned.toString()).to.equal(burnAmount.toString());
    });
  });

  describe("9. Minter quota enforcement", () => {
    it("should track minted amount against quota", async () => {
      const roles = await program.account.roleConfig.fetch(roleConfig);
      const minterInfo = roles.minters.find(
        (m: any) => m.address.toString() === minter.publicKey.toString()
      );
      expect(minterInfo).to.not.be.undefined;
      // Minted: 1000 + 1000 + 100 = 2100 tokens = 2_100_000_000 lamports
      expect(minterInfo.minted.toNumber()).to.be.greaterThan(0);
    });

    it("should fail to mint beyond quota", async () => {
      // Create a minter with a very small quota
      const smallMinter = Keypair.generate();
      await airdrop(provider.connection, smallMinter.publicKey);

      const smallQuota = new anchor.BN(100_000); // 0.1 tokens
      await addMinter(
        program,
        authority,
        stablecoinConfig,
        roleConfig,
        smallMinter.publicKey,
        smallQuota
      );

      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      // Try to mint more than quota
      const overQuotaAmount = new anchor.BN(1_000_000); // 1 token > 0.1 quota
      try {
        await mintTokens(
          program,
          smallMinter,
          stablecoinConfig,
          roleConfig,
          mint.publicKey,
          userAAta,
          overQuotaAmount
        );
        expect.fail("Mint beyond quota should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("MinterQuotaExceeded");
      }
    });
  });

  describe("10. Unauthorized access", () => {
    it("should fail when non-minter tries to mint", async () => {
      const userAAta = getAssociatedTokenAddressSync(
        mint.publicKey,
        userA.publicKey,
        false,
        TOKEN_2022_PROGRAM_ID
      );

      try {
        await mintTokens(
          program,
          unauthorized,
          stablecoinConfig,
          roleConfig,
          mint.publicKey,
          userAAta,
          MINT_AMOUNT
        );
        expect.fail("Unauthorized mint should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("MinterNotFound");
      }
    });

    it("should fail when non-authority tries to add minter", async () => {
      try {
        await addMinter(
          program,
          unauthorized,
          stablecoinConfig,
          roleConfig,
          Keypair.generate().publicKey,
          MINTER_QUOTA
        );
        expect.fail("Unauthorized add minter should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("Unauthorized");
      }
    });

    it("should fail when non-pauser tries to pause", async () => {
      try {
        await program.methods
          .pause()
          .accounts({
            pauser: unauthorized.publicKey,
            stablecoinConfig,
            roleConfig,
          })
          .signers([unauthorized])
          .rpc();
        expect.fail("Unauthorized pause should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("Unauthorized");
      }
    });
  });

  describe("11. SSS-2 instructions fail gracefully on SSS-1", () => {
    it("should fail when trying to add to blacklist on SSS-1 stablecoin", async () => {
      const targetAddress = Keypair.generate().publicKey;
      const [blacklistEntry] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("blacklist"),
          mint.publicKey.toBuffer(),
          targetAddress.toBuffer(),
        ],
        program.programId
      );

      try {
        await program.methods
          .addToBlacklist(targetAddress, "test reason")
          .accounts({
            blacklister: authority.publicKey,
            stablecoinConfig,
            roleConfig,
            blacklistEntry,
            systemProgram: SystemProgram.programId,
          })
          .signers([authority])
          .rpc();
        expect.fail("Blacklist on SSS-1 should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("ComplianceNotEnabled");
      }
    });

    it("should fail when trying to seize on SSS-1 stablecoin", async () => {
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

      try {
        await program.methods
          .seize(new anchor.BN(100_000_000))
          .accounts({
            seizer: authority.publicKey,
            stablecoinConfig,
            roleConfig,
            mint: mint.publicKey,
            fromTokenAccount: userAAta,
            toTokenAccount: userBAta,
            tokenProgram: TOKEN_2022_PROGRAM_ID,
          })
          .signers([authority])
          .rpc();
        expect.fail("Seize on SSS-1 should have failed");
      } catch (err: any) {
        expect(err.toString()).to.include("ComplianceNotEnabled");
      }
    });
  });
});
