export default function Home() {
  return (
    <div className="space-y-12">
      {/* Hero */}
      <section className="text-center py-16">
        <h1 className="text-5xl font-bold mb-4">
          <span className="text-solana-purple">Solana</span>{" "}
          <span className="text-solana-green">Stablecoin</span>{" "}
          <span className="text-white">Standard</span>
        </h1>
        <p className="text-gray-400 text-xl max-w-2xl mx-auto mb-8">
          Create and manage compliant stablecoins on Solana using Token-2022
          extensions. Two opinionated presets for every use case.
        </p>
        <div className="flex justify-center gap-4">
          <a
            href="/create"
            className="px-6 py-3 bg-solana-purple hover:bg-purple-700 rounded-lg font-semibold transition-colors"
          >
            Create Stablecoin
          </a>
          <a
            href="/manage"
            className="px-6 py-3 border border-solana-border hover:border-solana-green rounded-lg font-semibold transition-colors"
          >
            Manage Existing
          </a>
        </div>
      </section>

      {/* Presets */}
      <section className="grid md:grid-cols-2 gap-6">
        <div className="bg-solana-card border border-solana-border rounded-xl p-6 hover:border-blue-500 transition-colors">
          <div className="flex items-center gap-3 mb-4">
            <span className="px-3 py-1 bg-blue-500/20 text-blue-400 rounded-full text-sm font-semibold">
              SSS-1
            </span>
            <h3 className="text-xl font-bold">Minimal Preset</h3>
          </div>
          <p className="text-gray-400 mb-4">
            Basic stablecoin with mint/burn controls, freeze authority, and
            metadata. Perfect for internal tokens and simple use cases.
          </p>
          <ul className="space-y-2 text-sm text-gray-300">
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Mint Authority
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Freeze Authority
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Token Metadata
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Per-minter Quotas
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Global Pause
            </li>
          </ul>
        </div>

        <div className="bg-solana-card border border-solana-border rounded-xl p-6 hover:border-purple-500 transition-colors">
          <div className="flex items-center gap-3 mb-4">
            <span className="px-3 py-1 bg-purple-500/20 text-purple-400 rounded-full text-sm font-semibold">
              SSS-2
            </span>
            <h3 className="text-xl font-bold">Compliant Preset</h3>
          </div>
          <p className="text-gray-400 mb-4">
            Full compliance suite with blacklisting, seizure, and transfer
            controls. Meets GENIUS Act requirements.
          </p>
          <ul className="space-y-2 text-sm text-gray-300">
            <li className="flex items-center gap-2">
              <span className="text-solana-green">+</span> Everything in SSS-1
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-purple">+</span> Permanent Delegate
              (seize/burn)
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-purple">+</span> Transfer Hook
              (blacklist enforcement)
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-purple">+</span> Default Account
              State = Frozen
            </li>
            <li className="flex items-center gap-2">
              <span className="text-solana-purple">+</span> GENIUS Act Compliant
            </li>
          </ul>
        </div>
      </section>

      {/* Architecture */}
      <section className="bg-solana-card border border-solana-border rounded-xl p-8">
        <h2 className="text-2xl font-bold mb-6">Architecture</h2>
        <div className="grid md:grid-cols-3 gap-6">
          <div>
            <h4 className="text-solana-green font-semibold mb-2">
              On-Chain Programs
            </h4>
            <p className="text-sm text-gray-400">
              Two Anchor programs: sss-token (main program with runtime feature
              gating) and sss-transfer-hook (blacklist enforcement on every
              transfer).
            </p>
          </div>
          <div>
            <h4 className="text-solana-green font-semibold mb-2">
              TypeScript SDK
            </h4>
            <p className="text-sm text-gray-400">
              @stbr/sss-token SDK with SolanaStablecoin class, ComplianceModule,
              and OracleModule for building stablecoin applications.
            </p>
          </div>
          <div>
            <h4 className="text-solana-green font-semibold mb-2">
              Backend Services
            </h4>
            <p className="text-sm text-gray-400">
              Rust/Axum microservices for mint/burn lifecycle, compliance
              screening, event indexing, and webhook notifications.
            </p>
          </div>
        </div>
      </section>

      {/* Quick Stats */}
      <section className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[
          { label: "Token Standard", value: "Token-2022" },
          { label: "Presets", value: "2 (SSS-1, SSS-2)" },
          { label: "Extensions Used", value: "6" },
          { label: "Instructions", value: "13" },
        ].map((stat) => (
          <div
            key={stat.label}
            className="bg-solana-card border border-solana-border rounded-lg p-4 text-center"
          >
            <div className="text-2xl font-bold text-solana-green">
              {stat.value}
            </div>
            <div className="text-sm text-gray-400 mt-1">{stat.label}</div>
          </div>
        ))}
      </section>
    </div>
  );
}
