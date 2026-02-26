"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";
import { PublicKey } from "@solana/web3.js";

interface BlacklistEntry {
  address: string;
  reason: string;
  addedAt: string;
}

export default function CompliancePage() {
  const { connected } = useWallet();
  const [mintAddress, setMintAddress] = useState("");
  const [isLoaded, setIsLoaded] = useState(false);
  const [isCompliant, setIsCompliant] = useState(false);

  // Blacklist form state
  const [blacklistAddress, setBlacklistAddress] = useState("");
  const [blacklistReason, setBlacklistReason] = useState("");
  const [blacklistEntries, setBlacklistEntries] = useState<BlacklistEntry[]>([]);

  // Seize form state
  const [seizeFrom, setSeizeFrom] = useState("");
  const [seizeTo, setSeizeTo] = useState("");
  const [seizeAmount, setSeizeAmount] = useState("");

  const [status, setStatus] = useState<{
    type: "success" | "error" | "info";
    message: string;
  } | null>(null);

  const loadCompliance = () => {
    try {
      new PublicKey(mintAddress);
      // In a real app, fetch compliance config from RPC
      setIsCompliant(true);
      setIsLoaded(true);
      setBlacklistEntries([]);
    } catch {
      alert("Invalid mint address");
    }
  };

  const handleBlacklistAdd = async () => {
    if (!blacklistAddress || !blacklistReason) {
      setStatus({ type: "error", message: "Address and reason are required" });
      return;
    }
    try {
      new PublicKey(blacklistAddress);
    } catch {
      setStatus({ type: "error", message: "Invalid address" });
      return;
    }

    setStatus({ type: "info", message: "Submitting blacklist transaction..." });
    // In production, this would call the SDK
    setBlacklistEntries((prev) => [
      ...prev,
      {
        address: blacklistAddress,
        reason: blacklistReason,
        addedAt: new Date().toISOString(),
      },
    ]);
    setBlacklistAddress("");
    setBlacklistReason("");
    setStatus({ type: "success", message: "Address blacklisted successfully" });
  };

  const handleBlacklistRemove = async (address: string) => {
    setStatus({
      type: "info",
      message: "Submitting removal transaction...",
    });
    setBlacklistEntries((prev) => prev.filter((e) => e.address !== address));
    setStatus({
      type: "success",
      message: `Removed ${address.slice(0, 8)}... from blacklist`,
    });
  };

  const handleSeize = async () => {
    if (!seizeFrom || !seizeTo || !seizeAmount) {
      setStatus({ type: "error", message: "All seize fields are required" });
      return;
    }
    setStatus({ type: "info", message: "Submitting seize transaction..." });
    // In production, this would call the SDK
    setStatus({
      type: "success",
      message: `Seized ${seizeAmount} tokens from ${seizeFrom.slice(0, 8)}...`,
    });
    setSeizeFrom("");
    setSeizeTo("");
    setSeizeAmount("");
  };

  if (!connected) {
    return (
      <div className="max-w-3xl mx-auto">
        <h1 className="text-3xl font-bold mb-2">Compliance (SSS-2)</h1>
        <div className="bg-solana-card border border-solana-border rounded-xl p-8 text-center mt-8">
          <p className="text-gray-400">
            Connect your wallet to access compliance features.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Compliance (SSS-2)</h1>
        <p className="text-gray-400">
          Blacklist management and token seizure for compliant stablecoins.
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
            placeholder="Enter SSS-2 mint public key..."
            className="flex-1 bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
          />
          <button
            onClick={loadCompliance}
            className="px-6 py-2.5 bg-solana-purple hover:bg-purple-700 rounded-lg font-semibold transition-colors"
          >
            Load
          </button>
        </div>
      </div>

      {/* Status Message */}
      {status && (
        <div
          className={`p-4 rounded-lg border ${
            status.type === "success"
              ? "bg-green-500/10 border-green-500/30 text-green-400"
              : status.type === "error"
                ? "bg-red-500/10 border-red-500/30 text-red-400"
                : "bg-blue-500/10 border-blue-500/30 text-blue-400"
          }`}
        >
          {status.message}
        </div>
      )}

      {isLoaded && !isCompliant && (
        <div className="bg-solana-card border border-yellow-500/30 rounded-xl p-8 text-center">
          <p className="text-yellow-400 font-semibold mb-2">
            Compliance Not Enabled
          </p>
          <p className="text-gray-400">
            This stablecoin was created with SSS-1 preset. Blacklist and seize
            features require SSS-2.
          </p>
        </div>
      )}

      {isLoaded && isCompliant && (
        <>
          {/* Blacklist Management */}
          <div className="bg-solana-card border border-solana-border rounded-xl p-6 space-y-4">
            <h2 className="text-xl font-bold flex items-center gap-2">
              <span className="w-3 h-3 rounded-full bg-red-500" />
              Blacklist Management
            </h2>

            {/* Add to blacklist form */}
            <div className="space-y-3">
              <div>
                <label className="block text-sm text-gray-400 mb-1">
                  Address to Blacklist
                </label>
                <input
                  type="text"
                  value={blacklistAddress}
                  onChange={(e) => setBlacklistAddress(e.target.value)}
                  placeholder="Wallet address..."
                  className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">
                  Reason
                </label>
                <input
                  type="text"
                  value={blacklistReason}
                  onChange={(e) => setBlacklistReason(e.target.value)}
                  placeholder="e.g., OFAC sanctions list"
                  className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-red-500"
                />
              </div>
              <button
                onClick={handleBlacklistAdd}
                className="px-6 py-2.5 bg-red-600 hover:bg-red-700 rounded-lg font-semibold transition-colors"
              >
                Add to Blacklist
              </button>
            </div>

            {/* Blacklist entries table */}
            {blacklistEntries.length > 0 && (
              <div className="mt-4">
                <h3 className="text-sm font-semibold text-gray-300 mb-2">
                  Blacklisted Addresses ({blacklistEntries.length})
                </h3>
                <div className="space-y-2">
                  {blacklistEntries.map((entry) => (
                    <div
                      key={entry.address}
                      className="flex items-center justify-between bg-solana-dark rounded-lg p-3"
                    >
                      <div>
                        <p className="font-mono text-sm">
                          {entry.address.slice(0, 20)}...
                        </p>
                        <p className="text-xs text-gray-400">
                          {entry.reason} - {entry.addedAt}
                        </p>
                      </div>
                      <button
                        onClick={() => handleBlacklistRemove(entry.address)}
                        className="px-3 py-1 text-sm border border-red-500/50 text-red-400 rounded hover:bg-red-500/10 transition-colors"
                      >
                        Remove
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Seize Tokens */}
          <div className="bg-solana-card border border-solana-border rounded-xl p-6 space-y-4">
            <h2 className="text-xl font-bold flex items-center gap-2">
              <span className="w-3 h-3 rounded-full bg-yellow-500" />
              Seize Tokens
            </h2>
            <p className="text-sm text-gray-400">
              Transfer tokens from any account using the permanent delegate
              authority. This is an irreversible action.
            </p>

            <div className="grid md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">
                  From Address
                </label>
                <input
                  type="text"
                  value={seizeFrom}
                  onChange={(e) => setSeizeFrom(e.target.value)}
                  placeholder="Source wallet..."
                  className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-500"
                />
              </div>
              <div>
                <label className="block text-sm text-gray-400 mb-1">
                  To Treasury
                </label>
                <input
                  type="text"
                  value={seizeTo}
                  onChange={(e) => setSeizeTo(e.target.value)}
                  placeholder="Treasury wallet..."
                  className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-500"
                />
              </div>
            </div>
            <div>
              <label className="block text-sm text-gray-400 mb-1">
                Amount
              </label>
              <input
                type="text"
                value={seizeAmount}
                onChange={(e) => setSeizeAmount(e.target.value)}
                placeholder="Token amount to seize..."
                className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-yellow-500"
              />
            </div>
            <button
              onClick={handleSeize}
              className="px-6 py-2.5 bg-yellow-600 hover:bg-yellow-700 rounded-lg font-semibold transition-colors"
            >
              Seize Tokens
            </button>
          </div>
        </>
      )}
    </div>
  );
}
