"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";
import { OperationPanel } from "@/components/OperationPanel";

type Operation = "mint" | "burn" | "freeze" | "thaw" | "pause" | "unpause";

export default function ManagePage() {
  const { connected } = useWallet();
  const [mintAddress, setMintAddress] = useState("");
  const [activeOp, setActiveOp] = useState<Operation>("mint");
  const [isLoaded, setIsLoaded] = useState(false);
  const [configData, setConfigData] = useState<{
    name: string;
    symbol: string;
    decimals: number;
    paused: boolean;
    totalMinted: string;
    totalBurned: string;
    compliance: boolean;
  } | null>(null);

  const loadConfig = () => {
    try {
      new PublicKey(mintAddress);
      // In a real app, this would fetch from RPC
      setConfigData({
        name: "Example Stablecoin",
        symbol: "EUSD",
        decimals: 6,
        paused: false,
        totalMinted: "1,000,000.00",
        totalBurned: "50,000.00",
        compliance: false,
      });
      setIsLoaded(true);
    } catch {
      alert("Invalid mint address");
    }
  };

  if (!connected) {
    return (
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl font-bold mb-2">Manage Stablecoin</h1>
        <div className="bg-solana-card border border-solana-border rounded-xl p-8 text-center mt-8">
          <p className="text-gray-400">
            Connect your wallet to manage a stablecoin.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Manage Stablecoin</h1>
        <p className="text-gray-400">
          Perform operations on an existing stablecoin.
        </p>
      </div>

      {/* Mint Address Input */}
      <div className="bg-solana-card border border-solana-border rounded-xl p-6">
        <label className="block text-sm font-medium text-gray-300 mb-2">
          Stablecoin Mint Address
        </label>
        <div className="flex gap-3">
          <input
            type="text"
            value={mintAddress}
            onChange={(e) => {
              setMintAddress(e.target.value);
              setIsLoaded(false);
            }}
            placeholder="Enter mint public key..."
            className="flex-1 bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
          />
          <button
            onClick={loadConfig}
            className="px-6 py-2.5 bg-solana-purple hover:bg-purple-700 rounded-lg font-semibold transition-colors"
          >
            Load
          </button>
        </div>
      </div>

      {isLoaded && configData && (
        <>
          {/* Config Overview */}
          <div className="bg-solana-card border border-solana-border rounded-xl p-6">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-xl font-bold">{configData.name}</h2>
                <span className="text-gray-400">{configData.symbol}</span>
              </div>
              <div className="flex items-center gap-3">
                <span
                  className={`px-3 py-1 rounded-full text-sm font-semibold ${
                    configData.paused
                      ? "bg-red-500/20 text-red-400"
                      : "bg-green-500/20 text-green-400"
                  }`}
                >
                  {configData.paused ? "PAUSED" : "ACTIVE"}
                </span>
                <span
                  className={`px-3 py-1 rounded-full text-sm font-semibold ${
                    configData.compliance
                      ? "bg-purple-500/20 text-purple-400"
                      : "bg-blue-500/20 text-blue-400"
                  }`}
                >
                  {configData.compliance ? "SSS-2" : "SSS-1"}
                </span>
              </div>
            </div>
            <div className="grid grid-cols-3 gap-4 text-sm">
              <div>
                <span className="text-gray-400">Total Minted</span>
                <p className="text-lg font-semibold text-solana-green">
                  {configData.totalMinted}
                </p>
              </div>
              <div>
                <span className="text-gray-400">Total Burned</span>
                <p className="text-lg font-semibold text-red-400">
                  {configData.totalBurned}
                </p>
              </div>
              <div>
                <span className="text-gray-400">Decimals</span>
                <p className="text-lg font-semibold">{configData.decimals}</p>
              </div>
            </div>
          </div>

          {/* Operation Tabs */}
          <div className="flex gap-2 flex-wrap">
            {(
              [
                { key: "mint", label: "Mint", color: "green" },
                { key: "burn", label: "Burn", color: "red" },
                { key: "freeze", label: "Freeze", color: "blue" },
                { key: "thaw", label: "Thaw", color: "cyan" },
                { key: "pause", label: "Pause", color: "yellow" },
                { key: "unpause", label: "Unpause", color: "green" },
              ] as const
            ).map(({ key, label, color }) => (
              <button
                key={key}
                onClick={() => setActiveOp(key)}
                className={`px-4 py-2 rounded-lg text-sm font-semibold transition-all ${
                  activeOp === key
                    ? `bg-${color}-500/20 text-${color}-400 border border-${color}-500/50`
                    : "bg-solana-card border border-solana-border text-gray-400 hover:text-white"
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          {/* Operation Panel */}
          <OperationPanel
            operation={activeOp}
            mintAddress={mintAddress}
          />
        </>
      )}
    </div>
  );
}
