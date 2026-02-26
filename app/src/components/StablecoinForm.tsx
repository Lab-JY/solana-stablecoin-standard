"use client";

import { useState } from "react";
import { useWallet } from "@solana/wallet-adapter-react";

interface StablecoinFormProps {
  preset: "sss-1" | "sss-2";
}

export function StablecoinForm({ preset }: StablecoinFormProps) {
  const { publicKey } = useWallet();
  const [name, setName] = useState("");
  const [symbol, setSymbol] = useState("");
  const [decimals, setDecimals] = useState("6");
  const [uri, setUri] = useState("");
  const [defaultFrozen, setDefaultFrozen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [result, setResult] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!publicKey) return;

    setIsSubmitting(true);
    setResult(null);

    try {
      // Validate inputs
      if (!name || name.length > 32)
        throw new Error("Name is required (max 32 chars)");
      if (!symbol || symbol.length > 10)
        throw new Error("Symbol is required (max 10 chars)");
      const dec = parseInt(decimals);
      if (isNaN(dec) || dec < 0 || dec > 18)
        throw new Error("Decimals must be 0-18");

      // In production, this would call the SDK:
      // const stablecoin = await SolanaStablecoin.create(connection, {
      //   name, symbol, uri, decimals: dec,
      //   enablePermanentDelegate: preset === 'sss-2',
      //   enableTransferHook: preset === 'sss-2',
      //   defaultAccountFrozen: preset === 'sss-2' && defaultFrozen,
      // });

      setResult({
        type: "success",
        message: `Stablecoin "${name}" (${symbol}) created with ${preset.toUpperCase()} preset. Transaction would be submitted to the network.`,
      });
    } catch (err: any) {
      setResult({
        type: "error",
        message: err.message || "Failed to create stablecoin",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div className="bg-solana-card border border-solana-border rounded-xl p-6 space-y-4">
        <h2 className="text-lg font-semibold">Token Details</h2>

        <div className="grid md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Name <span className="text-red-400">*</span>
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Example USD"
              maxLength={32}
              className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
            />
            <span className="text-xs text-gray-500">{name.length}/32</span>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Symbol <span className="text-red-400">*</span>
            </label>
            <input
              type="text"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value.toUpperCase())}
              placeholder="e.g., EUSD"
              maxLength={10}
              className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
            />
            <span className="text-xs text-gray-500">{symbol.length}/10</span>
          </div>
        </div>

        <div className="grid md:grid-cols-2 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Decimals <span className="text-red-400">*</span>
            </label>
            <input
              type="number"
              value={decimals}
              onChange={(e) => setDecimals(e.target.value)}
              min={0}
              max={18}
              className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              Metadata URI
            </label>
            <input
              type="text"
              value={uri}
              onChange={(e) => setUri(e.target.value)}
              placeholder="https://..."
              className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
            />
          </div>
        </div>

        {preset === "sss-2" && (
          <div className="pt-4 border-t border-solana-border">
            <h3 className="text-sm font-semibold text-purple-400 mb-3">
              SSS-2 Compliance Options
            </h3>
            <label className="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={defaultFrozen}
                onChange={(e) => setDefaultFrozen(e.target.checked)}
                className="w-4 h-4 rounded border-gray-600 bg-solana-dark text-solana-purple focus:ring-solana-purple"
              />
              <div>
                <span className="text-sm font-medium">
                  Default Account State: Frozen
                </span>
                <p className="text-xs text-gray-400">
                  New token accounts will be frozen by default and require
                  explicit thawing (KYC flow).
                </p>
              </div>
            </label>
          </div>
        )}
      </div>

      {/* Result */}
      {result && (
        <div
          className={`p-4 rounded-lg border ${
            result.type === "success"
              ? "bg-green-500/10 border-green-500/30 text-green-400"
              : "bg-red-500/10 border-red-500/30 text-red-400"
          }`}
        >
          {result.message}
        </div>
      )}

      {/* Submit */}
      <button
        type="submit"
        disabled={isSubmitting || !name || !symbol}
        className={`w-full py-3 rounded-lg font-semibold transition-all ${
          isSubmitting || !name || !symbol
            ? "bg-gray-700 text-gray-500 cursor-not-allowed"
            : preset === "sss-2"
              ? "bg-solana-purple hover:bg-purple-700 text-white"
              : "bg-blue-600 hover:bg-blue-700 text-white"
        }`}
      >
        {isSubmitting
          ? "Creating..."
          : `Create ${preset.toUpperCase()} Stablecoin`}
      </button>
    </form>
  );
}
