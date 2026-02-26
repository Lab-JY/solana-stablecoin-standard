"use client";

import { useState } from "react";

type Operation = "mint" | "burn" | "freeze" | "thaw" | "pause" | "unpause";

interface OperationPanelProps {
  operation: Operation;
  mintAddress: string;
}

const operationConfig: Record<
  Operation,
  {
    title: string;
    description: string;
    color: string;
    fields: { name: string; label: string; placeholder: string; type: string }[];
  }
> = {
  mint: {
    title: "Mint Tokens",
    description: "Mint new tokens to a recipient address. Requires minter role with available quota.",
    color: "green",
    fields: [
      { name: "recipient", label: "Recipient Address", placeholder: "Wallet address...", type: "text" },
      { name: "amount", label: "Amount", placeholder: "1000000", type: "text" },
    ],
  },
  burn: {
    title: "Burn Tokens",
    description: "Burn tokens from a token account. Requires burner role.",
    color: "red",
    fields: [
      { name: "tokenAccount", label: "Token Account", placeholder: "Token account address...", type: "text" },
      { name: "amount", label: "Amount", placeholder: "1000000", type: "text" },
    ],
  },
  freeze: {
    title: "Freeze Account",
    description: "Freeze a token account to prevent all transfers. Requires authority or pauser role.",
    color: "blue",
    fields: [
      { name: "account", label: "Token Account to Freeze", placeholder: "Token account address...", type: "text" },
    ],
  },
  thaw: {
    title: "Thaw Account",
    description: "Unfreeze a frozen token account to re-enable transfers. Requires authority or pauser role.",
    color: "cyan",
    fields: [
      { name: "account", label: "Token Account to Thaw", placeholder: "Token account address...", type: "text" },
    ],
  },
  pause: {
    title: "Pause Stablecoin",
    description: "Globally pause all operations (mint, burn, transfer). Requires pauser role.",
    color: "yellow",
    fields: [],
  },
  unpause: {
    title: "Unpause Stablecoin",
    description: "Resume all operations after a pause. Requires pauser role.",
    color: "green",
    fields: [],
  },
};

export function OperationPanel({ operation, mintAddress }: OperationPanelProps) {
  const config = operationConfig[operation];
  const [fieldValues, setFieldValues] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [result, setResult] = useState<{
    type: "success" | "error";
    message: string;
  } | null>(null);

  const handleSubmit = async () => {
    setIsSubmitting(true);
    setResult(null);

    try {
      // Validate required fields
      for (const field of config.fields) {
        if (!fieldValues[field.name]) {
          throw new Error(`${field.label} is required`);
        }
      }

      // In production, this would call the SDK:
      // switch (operation) {
      //   case 'mint': await stablecoin.mint({ recipient, amount }); break;
      //   case 'burn': await stablecoin.burn({ amount, tokenAccount }); break;
      //   case 'freeze': await stablecoin.freezeAccount(account); break;
      //   case 'thaw': await stablecoin.thawAccount(account); break;
      //   case 'pause': await stablecoin.pause(); break;
      //   case 'unpause': await stablecoin.unpause(); break;
      // }

      setResult({
        type: "success",
        message: `${config.title} operation submitted successfully. Transaction would be confirmed on-chain.`,
      });
      setFieldValues({});
    } catch (err: any) {
      setResult({
        type: "error",
        message: err.message || `Failed to execute ${operation}`,
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="bg-solana-card border border-solana-border rounded-xl p-6 space-y-4">
      <div>
        <h2 className="text-xl font-bold">{config.title}</h2>
        <p className="text-sm text-gray-400 mt-1">{config.description}</p>
      </div>

      {config.fields.length > 0 ? (
        <div className="space-y-3">
          {config.fields.map((field) => (
            <div key={field.name}>
              <label className="block text-sm font-medium text-gray-300 mb-1">
                {field.label}
              </label>
              <input
                type={field.type}
                value={fieldValues[field.name] || ""}
                onChange={(e) =>
                  setFieldValues((prev) => ({
                    ...prev,
                    [field.name]: e.target.value,
                  }))
                }
                placeholder={field.placeholder}
                className="w-full bg-solana-dark border border-solana-border rounded-lg px-4 py-2.5 text-white placeholder-gray-500 focus:outline-none focus:border-solana-purple"
              />
            </div>
          ))}
        </div>
      ) : (
        <div className="py-4 text-center text-gray-400 text-sm">
          This operation does not require additional parameters.
        </div>
      )}

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

      <button
        onClick={handleSubmit}
        disabled={isSubmitting}
        className={`w-full py-3 rounded-lg font-semibold transition-all ${
          isSubmitting
            ? "bg-gray-700 text-gray-500 cursor-not-allowed"
            : "bg-solana-purple hover:bg-purple-700 text-white"
        }`}
      >
        {isSubmitting ? "Submitting..." : `Execute ${config.title}`}
      </button>

      <p className="text-xs text-gray-500 text-center">
        Target mint: {mintAddress.slice(0, 16)}...{mintAddress.slice(-8)}
      </p>
    </div>
  );
}
