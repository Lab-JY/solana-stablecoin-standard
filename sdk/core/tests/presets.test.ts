import { expect } from "chai";
import { Presets, getPresetConfig } from "../src/presets";

describe("Presets", () => {
  describe("Presets enum", () => {
    it("should have SSS_1 preset", () => {
      expect(Presets.SSS_1).to.equal("sss-1");
    });

    it("should have SSS_2 preset", () => {
      expect(Presets.SSS_2).to.equal("sss-2");
    });
  });

  describe("getPresetConfig", () => {
    describe("SSS-1 preset", () => {
      const config = getPresetConfig(Presets.SSS_1);

      it("should return 6 decimals", () => {
        expect(config.decimals).to.equal(6);
      });

      it("should have permanent delegate disabled", () => {
        expect(config.enablePermanentDelegate).to.be.false;
      });

      it("should have transfer hook disabled", () => {
        expect(config.enableTransferHook).to.be.false;
      });

      it("should have default account frozen disabled", () => {
        expect(config.defaultAccountFrozen).to.be.false;
      });

      it("should include default name and symbol", () => {
        expect(config.name).to.be.a("string").and.not.empty;
        expect(config.symbol).to.be.a("string").and.not.empty;
      });
    });

    describe("SSS-2 preset", () => {
      const config = getPresetConfig(Presets.SSS_2);

      it("should return 6 decimals", () => {
        expect(config.decimals).to.equal(6);
      });

      it("should have permanent delegate enabled", () => {
        expect(config.enablePermanentDelegate).to.be.true;
      });

      it("should have transfer hook enabled", () => {
        expect(config.enableTransferHook).to.be.true;
      });

      it("should have default account frozen disabled by default", () => {
        expect(config.defaultAccountFrozen).to.be.false;
      });

      it("should include default name and symbol", () => {
        expect(config.name).to.be.a("string").and.not.empty;
        expect(config.symbol).to.be.a("string").and.not.empty;
      });
    });

    it("should throw for unknown presets", () => {
      expect(() => getPresetConfig("unknown" as Presets)).to.throw(
        "Unknown preset"
      );
    });

    it("SSS-2 should differ from SSS-1 in compliance features", () => {
      const sss1 = getPresetConfig(Presets.SSS_1);
      const sss2 = getPresetConfig(Presets.SSS_2);

      expect(sss1.enablePermanentDelegate).to.not.equal(
        sss2.enablePermanentDelegate
      );
      expect(sss1.enableTransferHook).to.not.equal(sss2.enableTransferHook);
    });
  });
});
