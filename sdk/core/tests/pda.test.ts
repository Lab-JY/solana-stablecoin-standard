import { expect } from "chai";
import { PublicKey } from "@solana/web3.js";
import {
  deriveStablecoinConfig,
  deriveRoleConfig,
  deriveBlacklistEntry,
} from "../src/pda";

const PROGRAM_ID = new PublicKey(
  "AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE"
);

describe("PDA Derivation", () => {
  const mint = PublicKey.unique();

  describe("deriveStablecoinConfig", () => {
    it("should derive a valid PDA with correct seeds", () => {
      const [pda, bump] = deriveStablecoinConfig(PROGRAM_ID, mint);
      expect(pda).to.be.instanceOf(PublicKey);
      expect(bump).to.be.a("number");
      expect(bump).to.be.gte(0).and.lte(255);
    });

    it("should derive the same PDA for the same inputs", () => {
      const [pda1] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [pda2] = deriveStablecoinConfig(PROGRAM_ID, mint);
      expect(pda1.equals(pda2)).to.be.true;
    });

    it("should derive different PDAs for different mints", () => {
      const mint2 = PublicKey.unique();
      const [pda1] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [pda2] = deriveStablecoinConfig(PROGRAM_ID, mint2);
      expect(pda1.equals(pda2)).to.be.false;
    });

    it("should derive different PDAs for different program IDs", () => {
      const otherProgram = PublicKey.unique();
      const [pda1] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [pda2] = deriveStablecoinConfig(otherProgram, mint);
      expect(pda1.equals(pda2)).to.be.false;
    });

    it("should match manual PDA derivation", () => {
      const [expected] = PublicKey.findProgramAddressSync(
        [Buffer.from("stablecoin"), mint.toBuffer()],
        PROGRAM_ID
      );
      const [pda] = deriveStablecoinConfig(PROGRAM_ID, mint);
      expect(pda.equals(expected)).to.be.true;
    });
  });

  describe("deriveRoleConfig", () => {
    it("should derive a valid PDA with correct seeds", () => {
      const [stablecoinConfig] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [pda, bump] = deriveRoleConfig(PROGRAM_ID, stablecoinConfig);
      expect(pda).to.be.instanceOf(PublicKey);
      expect(bump).to.be.a("number");
      expect(bump).to.be.gte(0).and.lte(255);
    });

    it("should be deterministic", () => {
      const [stablecoinConfig] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [pda1] = deriveRoleConfig(PROGRAM_ID, stablecoinConfig);
      const [pda2] = deriveRoleConfig(PROGRAM_ID, stablecoinConfig);
      expect(pda1.equals(pda2)).to.be.true;
    });

    it("should match manual PDA derivation", () => {
      const [stablecoinConfig] = deriveStablecoinConfig(PROGRAM_ID, mint);
      const [expected] = PublicKey.findProgramAddressSync(
        [Buffer.from("roles"), stablecoinConfig.toBuffer()],
        PROGRAM_ID
      );
      const [pda] = deriveRoleConfig(PROGRAM_ID, stablecoinConfig);
      expect(pda.equals(expected)).to.be.true;
    });
  });

  describe("deriveBlacklistEntry", () => {
    const address = PublicKey.unique();

    it("should derive a valid PDA with correct seeds", () => {
      const [pda, bump] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      expect(pda).to.be.instanceOf(PublicKey);
      expect(bump).to.be.a("number");
      expect(bump).to.be.gte(0).and.lte(255);
    });

    it("should be deterministic", () => {
      const [pda1] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      const [pda2] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      expect(pda1.equals(pda2)).to.be.true;
    });

    it("should produce different PDAs for different addresses", () => {
      const address2 = PublicKey.unique();
      const [pda1] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      const [pda2] = deriveBlacklistEntry(PROGRAM_ID, mint, address2);
      expect(pda1.equals(pda2)).to.be.false;
    });

    it("should produce different PDAs for different mints", () => {
      const mint2 = PublicKey.unique();
      const [pda1] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      const [pda2] = deriveBlacklistEntry(PROGRAM_ID, mint2, address);
      expect(pda1.equals(pda2)).to.be.false;
    });

    it("should match manual PDA derivation", () => {
      const [expected] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("blacklist"),
          mint.toBuffer(),
          address.toBuffer(),
        ],
        PROGRAM_ID
      );
      const [pda] = deriveBlacklistEntry(PROGRAM_ID, mint, address);
      expect(pda.equals(expected)).to.be.true;
    });
  });
});
