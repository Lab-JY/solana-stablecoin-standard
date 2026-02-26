"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { StablecoinForm } from "@/components/StablecoinForm";

type Preset = "sss-1" | "sss-2";

export default function CreatePage() {
  const { connected } = useWallet();
  const [selectedPreset, setSelectedPreset] = useState<Preset>("sss-1");

  return (
    <div className="max-w-3xl mx-auto space-y-8">
      <div>
        <h1 className="text-3xl font-bold mb-2">Create Stablecoin</h1>
        <p className="text-gray-400">
          Deploy a new stablecoin using the Solana Stablecoin Standard.
        </p>
      </div>

      {!connected ? (
        <div className="bg-solana-card border border-solana-border rounded-xl p-8 text-center">
          <p className="text-gray-400 mb-4">
            Connect your wallet to create a stablecoin.
          </p>
          <p className="text-sm text-gray-500">
            Click the wallet button in the top-right corner.
          </p>
        </div>
      ) : (
        <>
          {/* Preset Selection */}
          <div className="space-y-4">
            <h2 className="text-lg font-semibold">Select Preset</h2>
            <div className="grid md:grid-cols-2 gap-4">
              <button
                onClick={() => setSelectedPreset("sss-1")}
                className={`p-4 rounded-xl border text-left transition-all ${
                  selectedPreset === "sss-1"
                    ? "border-blue-500 bg-blue-500/10"
                    : "border-solana-border bg-solana-card hover:border-gray-500"
                }`}
              >
                <div className="flex items-center gap-2 mb-2">
                  <span className="px-2 py-0.5 bg-blue-500/20 text-blue-400 rounded text-xs font-semibold">
                    SSS-1
                  </span>
                  <span className="font-semibold">Minimal</span>
                </div>
                <p className="text-sm text-gray-400">
                  Basic mint/burn, freeze, metadata. No compliance features.
                </p>
              </button>

              <button
                onClick={() => setSelectedPreset("sss-2")}
                className={`p-4 rounded-xl border text-left transition-all ${
                  selectedPreset === "sss-2"
                    ? "border-purple-500 bg-purple-500/10"
                    : "border-solana-border bg-solana-card hover:border-gray-500"
                }`}
              >
                <div className="flex items-center gap-2 mb-2">
                  <span className="px-2 py-0.5 bg-purple-500/20 text-purple-400 rounded text-xs font-semibold">
                    SSS-2
                  </span>
                  <span className="font-semibold">Compliant</span>
                </div>
                <p className="text-sm text-gray-400">
                  Full compliance: blacklist, seize, transfer hook, default
                  frozen.
                </p>
              </button>
            </div>
          </div>

          {/* Form */}
          <StablecoinForm preset={selectedPreset} />

          {/* Extensions Preview */}
          <div className="bg-solana-card border border-solana-border rounded-xl p-6">
            <h3 className="font-semibold mb-3">
              Token-2022 Extensions (
              {selectedPreset === "sss-1" ? "SSS-1" : "SSS-2"})
            </h3>
            <div className="grid grid-cols-2 gap-2 text-sm">
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-solana-green" />
                MintAuthority
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-solana-green" />
                FreezeAuthority
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-solana-green" />
                MetadataPointer
              </div>
              <div className="flex items-center gap-2">
                <span className="w-2 h-2 rounded-full bg-solana-green" />
                TokenMetadata
              </div>
              {selectedPreset === "sss-2" && (
                <>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-solana-purple" />
                    PermanentDelegate
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-solana-purple" />
                    TransferHook
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="w-2 h-2 rounded-full bg-solana-purple" />
                    DefaultAccountState
                  </div>
                </>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
