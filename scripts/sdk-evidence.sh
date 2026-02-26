#!/usr/bin/env bash
set -euo pipefail

unset http_proxy https_proxy all_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY

ART_DIR="docs/deployment-artifacts/$(date +%Y%m%d-%H%M%S)-sdk-localnet"
KEYPAIR_PATH="/tmp/sss-sdk-authority-$(date +%s).json"
LEDGER_DIR="/tmp/sss-sdk-ledger-$(date +%s)"

mkdir -p "$ART_DIR"

solana-keygen new --no-bip39-passphrase --silent -o "$KEYPAIR_PATH" > "$ART_DIR/keygen.txt" 2>&1

solana-test-validator --reset --ledger "$LEDGER_DIR" --rpc-port 8899 > "$ART_DIR/validator.log" 2>&1 &
VALIDATOR_PID=$!
echo "$VALIDATOR_PID" > "$ART_DIR/validator.pid"

cleanup() {
  if kill -0 "$VALIDATOR_PID" 2>/dev/null; then
    kill "$VALIDATOR_PID" || true
    wait "$VALIDATOR_PID" 2>/dev/null || true
  fi
  rm -f "$KEYPAIR_PATH"
}
trap cleanup EXIT

for _ in $(seq 1 60); do
  if solana -u http://127.0.0.1:8899 block-height > "$ART_DIR/block-height.txt" 2>&1; then
    break
  fi
  sleep 1
done

solana -u http://127.0.0.1:8899 -k "$KEYPAIR_PATH" airdrop 20 > "$ART_DIR/airdrop.txt" 2>&1
solana -u http://127.0.0.1:8899 -k "$KEYPAIR_PATH" balance > "$ART_DIR/balance.txt" 2>&1

anchor deploy --provider.cluster localnet --provider.wallet "$KEYPAIR_PATH" -p sss_transfer_hook > "$ART_DIR/deploy-transfer-hook.txt" 2>&1
anchor deploy --provider.cluster localnet --provider.wallet "$KEYPAIR_PATH" -p sss_token > "$ART_DIR/deploy-sss-token.txt" 2>&1

SSS_TOKEN_PID="$(solana address -k target/deploy/sss_token-keypair.json)"
SSS_HOOK_PID="$(solana address -k target/deploy/sss_transfer_hook-keypair.json)"

cat > "$ART_DIR/sdk-runtime-evidence.ts" <<'TS'
import fs from "node:fs";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { AnchorProvider, BN, Idl, Wallet } from "@coral-xyz/anchor";
import { createAssociatedTokenAccountIdempotent, TOKEN_2022_PROGRAM_ID } from "@solana/spl-token";
import { SolanaStablecoin, Presets, getPresetConfig, MinterAction } from "../../../sdk/core/src";

async function main() {
  const rpcUrl = process.env.RPC_URL!;
  const keypairPath = process.env.AUTHORITY_KEYPAIR!;
  const programId = new PublicKey(process.env.SSS_TOKEN_PROGRAM_ID!);
  const transferHookProgramId = new PublicKey(process.env.SSS_TRANSFER_HOOK_PROGRAM_ID!);

  const secret = JSON.parse(fs.readFileSync(keypairPath, "utf-8"));
  const authority = Keypair.fromSecretKey(Uint8Array.from(secret));
  const connection = new Connection(rpcUrl, "confirmed");
  const wallet = new Wallet(authority);
  const provider = new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
    preflightCommitment: "confirmed",
  });

  const idl = JSON.parse(fs.readFileSync("target/idl/sss_token.json", "utf-8")) as Idl;

  const evidence: any = {
    cluster: rpcUrl,
    programId: programId.toBase58(),
    transferHookProgramId: transferHookProgramId.toBase58(),
    authority: authority.publicKey.toBase58(),
  };

  const sss1Mint = Keypair.generate();
  const sss1Params = {
    ...getPresetConfig(Presets.SSS_1),
    name: "SDK SSS1",
    symbol: "SDK1",
    uri: "https://example.com/sdk1.json",
  };
  const sss1Created = await SolanaStablecoin.create(
    provider,
    idl,
    sss1Params,
    sss1Mint,
    programId,
    transferHookProgramId
  );
  evidence.sss1 = {
    mint: sss1Created.stablecoin.mintAddress.toBase58(),
    initSignature: sss1Created.signature,
  };

  const customMint = Keypair.generate();
  const customParams = {
    name: "SDK Custom",
    symbol: "SDKC",
    uri: "https://example.com/sdkc.json",
    decimals: 6,
    enablePermanentDelegate: true,
    enableTransferHook: false,
    defaultAccountFrozen: false,
  };
  const customCreated = await SolanaStablecoin.create(
    provider,
    idl,
    customParams,
    customMint,
    programId,
    transferHookProgramId
  );

  const quota = new BN(5_000_000);
  const customAddMinterSig = await customCreated.stablecoin.updateMinter(
    { action: MinterAction.Add, address: authority.publicKey, quota },
    authority
  );
  await createAssociatedTokenAccountIdempotent(
    connection,
    authority,
    customCreated.stablecoin.mintAddress,
    authority.publicKey,
    { commitment: "confirmed" },
    TOKEN_2022_PROGRAM_ID
  );
  const customMintSig = await customCreated.stablecoin.mint(
    { recipient: authority.publicKey, amount: new BN(1_000_000) },
    authority
  );
  const customSupply = await customCreated.stablecoin.getTotalSupply();

  evidence.custom = {
    mint: customCreated.stablecoin.mintAddress.toBase58(),
    initSignature: customCreated.signature,
    addMinterSignature: customAddMinterSig,
    mintSignature: customMintSig,
    totalSupply: customSupply.toString(),
  };

  const sss2Mint = Keypair.generate();
  const sss2Params = {
    ...getPresetConfig(Presets.SSS_2),
    name: "SDK SSS2",
    symbol: "SDK2",
    uri: "https://example.com/sdk2.json",
  };
  const sss2Created = await SolanaStablecoin.create(
    provider,
    idl,
    sss2Params,
    sss2Mint,
    programId,
    transferHookProgramId
  );

  const sss2AddMinterSig = await sss2Created.stablecoin.updateMinter(
    { action: MinterAction.Add, address: authority.publicKey, quota },
    authority
  );
  await createAssociatedTokenAccountIdempotent(
    connection,
    authority,
    sss2Created.stablecoin.mintAddress,
    authority.publicKey,
    { commitment: "confirmed" },
    TOKEN_2022_PROGRAM_ID
  );
  const sss2MintSig = await sss2Created.stablecoin.mint(
    { recipient: authority.publicKey, amount: new BN(1_000_000) },
    authority
  );
  const sss2Supply = await sss2Created.stablecoin.getTotalSupply();

  const blacklistAddSig = await sss2Created.stablecoin.compliance.blacklistAdd(
    authority.publicKey,
    "SDK runtime evidence",
    authority
  );
  const isBlacklistedAfterAdd = await sss2Created.stablecoin.compliance.isBlacklisted(authority.publicKey);
  const blacklistRemoveSig = await sss2Created.stablecoin.compliance.blacklistRemove(
    authority.publicKey,
    authority
  );
  const isBlacklistedAfterRemove = await sss2Created.stablecoin.compliance.isBlacklisted(authority.publicKey);

  let sss1ComplianceFailure: string | null = null;
  try {
    await sss1Created.stablecoin.compliance.blacklistAdd(
      authority.publicKey,
      "should fail",
      authority
    );
  } catch (err: any) {
    sss1ComplianceFailure = String(err?.message ?? err);
  }

  evidence.sss2 = {
    mint: sss2Created.stablecoin.mintAddress.toBase58(),
    initSignature: sss2Created.signature,
    addMinterSignature: sss2AddMinterSig,
    mintSignature: sss2MintSig,
    totalSupply: sss2Supply.toString(),
    blacklistAddSignature: blacklistAddSig,
    blacklistRemoveSignature: blacklistRemoveSig,
    isBlacklistedAfterAdd,
    isBlacklistedAfterRemove,
  };
  evidence.sss1ComplianceFailure = sss1ComplianceFailure;

  const loaded = await SolanaStablecoin.load(
    provider,
    idl,
    customCreated.stablecoin.mintAddress,
    programId,
    transferHookProgramId
  );
  evidence.loadCheck = {
    loadedMint: loaded.mintAddress.toBase58(),
    matchesCustomMint: loaded.mintAddress.equals(customCreated.stablecoin.mintAddress),
  };

  console.log(JSON.stringify(evidence, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
TS

RPC_URL="http://127.0.0.1:8899" \
AUTHORITY_KEYPAIR="$KEYPAIR_PATH" \
SSS_TOKEN_PROGRAM_ID="$SSS_TOKEN_PID" \
SSS_TRANSFER_HOOK_PROGRAM_ID="$SSS_HOOK_PID" \
TS_NODE_COMPILER_OPTIONS='{"module":"commonjs","moduleResolution":"node"}' \
  npx ts-node --transpile-only "$ART_DIR/sdk-runtime-evidence.ts" > "$ART_DIR/sdk-runtime.json" 2> "$ART_DIR/sdk-runtime.stderr"

npm --workspace sdk/core test > "$ART_DIR/sdk-core-tests.txt" 2>&1

cat > "$ART_DIR/summary.txt" <<EOF
SDK runtime evidence (localnet)

Authority: $(solana address -k "$KEYPAIR_PATH")
Token program: $SSS_TOKEN_PID
Transfer-hook program: $SSS_HOOK_PID

Artifacts:
- sdk-runtime.json: preset init (SSS-1, SSS-2), custom init, mint/supply ops, compliance add/remove runtime, graceful failure on SSS-1 compliance op, load() check
- sdk-core-tests.txt: unit tests for preset/PDA helpers
- deploy-transfer-hook.txt / deploy-sss-token.txt: local deployments
- validator.log: local validator logs
EOF

echo "$ART_DIR"
