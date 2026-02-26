#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required"
  exit 1
fi

if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI is required"
  exit 1
fi

RPC_URL="${RPC_URL:-https://api.devnet.solana.com}"
KEYPAIR_PATH="${KEYPAIR_PATH:-$HOME/.config/solana/id.json}"
TREASURY_ADDRESS="${TREASURY_ADDRESS:-}"

if [[ -z "$TREASURY_ADDRESS" ]]; then
  echo "TREASURY_ADDRESS is required"
  exit 1
fi

OPERATOR_ADDRESS="$(solana-keygen pubkey "$KEYPAIR_PATH")"

ARTIFACT_DIR="docs/deployment-artifacts/$(date +%Y%m%d-%H%M%S)-smoke"
mkdir -p "$ARTIFACT_DIR"
TX_LOG="$ARTIFACT_DIR/transactions.txt"

run_cli() {
  NO_COLOR=1 cargo run -q -p sss-token-cli -- \
    --url "$RPC_URL" \
    --keypair "$KEYPAIR_PATH" \
    "$@"
}

extract_field() {
  local output="$1"
  local label="$2"
  echo "$output" | sed -n "s/^  ${label}: //p" | head -n1
}

extract_tx() {
  local output="$1"
  echo "$output" | sed -n 's/^Transaction: //p' | head -n1
}

record_tx() {
  local label="$1"
  local output="$2"
  local tx
  tx="$(extract_tx "$output")"
  if [[ -n "$tx" ]]; then
    echo "${label}=${tx}" | tee -a "$TX_LOG"
  fi
}

echo "==> Init SSS-1"
OUT_INIT_1="$(run_cli init --preset sss-1)"
echo "$OUT_INIT_1" | tee "$ARTIFACT_DIR/init-sss1.log"
MINT_SSS1="$(extract_field "$OUT_INIT_1" "Mint")"
echo "SSS1_MINT=$MINT_SSS1" | tee -a "$TX_LOG"
record_tx "sss1_init_tx" "$OUT_INIT_1"

OUT_SSS1_MINTER="$(run_cli --mint "$MINT_SSS1" minters add "$OPERATOR_ADDRESS" --quota 1000000000000)"
echo "$OUT_SSS1_MINTER" | tee "$ARTIFACT_DIR/sss1-minter-add.log"
record_tx "sss1_add_minter_tx" "$OUT_SSS1_MINTER"

OUT_SSS1_MINT="$(run_cli --mint "$MINT_SSS1" mint "$OPERATOR_ADDRESS" 1000000)"
echo "$OUT_SSS1_MINT" | tee "$ARTIFACT_DIR/sss1-mint.log"
record_tx "sss1_mint_tx" "$OUT_SSS1_MINT"

echo "==> Init SSS-2"
OUT_INIT_2="$(run_cli init --preset sss-2)"
echo "$OUT_INIT_2" | tee "$ARTIFACT_DIR/init-sss2.log"
MINT_SSS2="$(extract_field "$OUT_INIT_2" "Mint")"
echo "SSS2_MINT=$MINT_SSS2" | tee -a "$TX_LOG"
record_tx "sss2_init_tx" "$OUT_INIT_2"

OUT_SSS2_MINTER="$(run_cli --mint "$MINT_SSS2" minters add "$OPERATOR_ADDRESS" --quota 1000000000000)"
echo "$OUT_SSS2_MINTER" | tee "$ARTIFACT_DIR/sss2-minter-add.log"
record_tx "sss2_add_minter_tx" "$OUT_SSS2_MINTER"

OUT_SSS2_MINT_FROM="$(run_cli --mint "$MINT_SSS2" mint "$OPERATOR_ADDRESS" 1000000)"
echo "$OUT_SSS2_MINT_FROM" | tee "$ARTIFACT_DIR/sss2-mint-operator.log"
record_tx "sss2_mint_operator_tx" "$OUT_SSS2_MINT_FROM"

OUT_SSS2_MINT_TO="$(run_cli --mint "$MINT_SSS2" mint "$TREASURY_ADDRESS" 1)"
echo "$OUT_SSS2_MINT_TO" | tee "$ARTIFACT_DIR/sss2-mint-treasury.log"
record_tx "sss2_mint_treasury_tx" "$OUT_SSS2_MINT_TO"

OUT_BL_ADD="$(run_cli --mint "$MINT_SSS2" blacklist add "$OPERATOR_ADDRESS" --reason "smoke-test")"
echo "$OUT_BL_ADD" | tee "$ARTIFACT_DIR/sss2-blacklist-add.log"
record_tx "sss2_blacklist_add_tx" "$OUT_BL_ADD"

OUT_SEIZE="$(run_cli --mint "$MINT_SSS2" seize "$OPERATOR_ADDRESS" --to "$TREASURY_ADDRESS")"
echo "$OUT_SEIZE" | tee "$ARTIFACT_DIR/sss2-seize.log"
record_tx "sss2_seize_tx" "$OUT_SEIZE"

OUT_BL_REMOVE="$(run_cli --mint "$MINT_SSS2" blacklist remove "$OPERATOR_ADDRESS")"
echo "$OUT_BL_REMOVE" | tee "$ARTIFACT_DIR/sss2-blacklist-remove.log"
record_tx "sss2_blacklist_remove_tx" "$OUT_BL_REMOVE"

echo "Smoke artifacts saved in $ARTIFACT_DIR"
echo "Update docs/DEPLOYMENT.md with contents from $TX_LOG"
