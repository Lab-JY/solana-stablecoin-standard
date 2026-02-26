import { InitializeParams } from "./types";

/** Supported stablecoin presets */
export enum Presets {
  /** Minimal stablecoin: mint, burn, freeze, thaw, pause */
  SSS_1 = "sss-1",
  /** Compliant stablecoin: SSS-1 + permanent delegate, transfer hook, blacklist, seize */
  SSS_2 = "sss-2",
}

/**
 * Returns the default InitializeParams for a given preset.
 * Callers should override name/symbol/uri before using.
 */
export function getPresetConfig(preset: Presets): InitializeParams {
  switch (preset) {
    case Presets.SSS_1:
      return {
        name: "My Stablecoin",
        symbol: "MUSD",
        uri: "",
        decimals: 6,
        enablePermanentDelegate: false,
        enableTransferHook: false,
        defaultAccountFrozen: false,
      };

    case Presets.SSS_2:
      return {
        name: "My Compliant Stablecoin",
        symbol: "MUSD",
        uri: "",
        decimals: 6,
        enablePermanentDelegate: true,
        enableTransferHook: true,
        defaultAccountFrozen: false,
      };

    default:
      throw new Error(`Unknown preset: ${preset}`);
  }
}
