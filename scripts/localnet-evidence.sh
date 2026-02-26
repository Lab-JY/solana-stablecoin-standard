#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is required"
    exit 1
  fi
}

need_cmd anchor
need_cmd cargo
need_cmd solana
need_cmd solana-keygen
need_cmd solana-test-validator

# Local RPC calls should never go through external proxies.
unset http_proxy https_proxy all_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY

RPC_URL="${RPC_URL:-http://127.0.0.1:8899}"
ARTIFACT_DIR="docs/deployment-artifacts/$(date +%Y%m%d-%H%M%S)-localnet"
mkdir -p "$ARTIFACT_DIR"
TX_LOG="$ARTIFACT_DIR/transactions.txt"

KEYPAIR_PATH="${KEYPAIR_PATH:-$ARTIFACT_DIR/operator-keypair.json}"
TREASURY_KEYPAIR_PATH="${TREASURY_KEYPAIR_PATH:-$ARTIFACT_DIR/treasury-keypair.json}"

if [[ ! -f "$KEYPAIR_PATH" ]]; then
  solana-keygen new --no-bip39-passphrase -f -o "$KEYPAIR_PATH" >/dev/null
fi
if [[ ! -f "$TREASURY_KEYPAIR_PATH" ]]; then
  solana-keygen new --no-bip39-passphrase -f -o "$TREASURY_KEYPAIR_PATH" >/dev/null
fi

OPERATOR_ADDRESS="$(solana-keygen pubkey "$KEYPAIR_PATH")"
TREASURY_ADDRESS="$(solana-keygen pubkey "$TREASURY_KEYPAIR_PATH")"

echo "operator=$OPERATOR_ADDRESS" | tee -a "$TX_LOG"
echo "treasury=$TREASURY_ADDRESS" | tee -a "$TX_LOG"

VALIDATOR_LOG="$ARTIFACT_DIR/validator.log"
VALIDATOR_LEDGER="$ARTIFACT_DIR/test-ledger"

cleanup() {
  if [[ -n "${VALIDATOR_PID:-}" ]]; then
    kill "$VALIDATOR_PID" >/dev/null 2>&1 || true
    wait "$VALIDATOR_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

start_validator() {
  echo "==> Starting local validator"
  solana-test-validator --reset --ledger "$VALIDATOR_LEDGER" >"$VALIDATOR_LOG" 2>&1 &
  VALIDATOR_PID=$!

  for _ in $(seq 1 60); do
    if solana -u "$RPC_URL" block-height >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done

  echo "local validator failed to start"
  exit 1
}

record_tx() {
  local label="$1"
  local output="$2"
  local tx
  tx="$(echo "$output" | sed -n 's/^Transaction: //p' | head -n1)"
  if [[ -n "$tx" ]]; then
    echo "${label}=${tx}" | tee -a "$TX_LOG"
  fi
}

extract_field() {
  local output="$1"
  local label="$2"
  echo "$output" | sed -n "s/^  ${label}: //p" | head -n1
}

run_cli() {
  NO_COLOR=1 cargo run -q -p sss-token-cli -- \
    --url "$RPC_URL" \
    --keypair "$KEYPAIR_PATH" \
    "$@"
}

run_cli_retry() {
  local attempts="$1"
  shift
  local n=1
  local out
  while (( n <= attempts )); do
    if out="$(run_cli "$@" 2>&1)"; then
      printf '%s\n' "$out"
      return 0
    fi
    if (( n == attempts )); then
      printf '%s\n' "$out"
      return 1
    fi
    echo "Retrying command (attempt $((n + 1))/$attempts): $*" >&2
    sleep 2
    n=$((n + 1))
  done
}

start_validator

echo "==> Funding operator"
AIRDROP_OUT="$(solana -u "$RPC_URL" airdrop 200 "$OPERATOR_ADDRESS")"
echo "$AIRDROP_OUT" | tee "$ARTIFACT_DIR/airdrop.log"
AIRDROP_SIG="$(echo "$AIRDROP_OUT" | sed -n 's/^Signature: //p' | head -n1)"
if [[ -n "$AIRDROP_SIG" ]]; then
  echo "fund_operator_airdrop_tx=$AIRDROP_SIG" | tee -a "$TX_LOG"
fi

echo "==> Building programs"
anchor build --no-idl 2>&1 | tee "$ARTIFACT_DIR/build.log"

echo "==> Deploying transfer hook"
anchor deploy --provider.cluster localnet --provider.wallet "$KEYPAIR_PATH" -p sss_transfer_hook \
  2>&1 | tee "$ARTIFACT_DIR/deploy-transfer-hook.log"

echo "==> Deploying token"
anchor deploy --provider.cluster localnet --provider.wallet "$KEYPAIR_PATH" -p sss_token \
  2>&1 | tee "$ARTIFACT_DIR/deploy-token.log"

echo "cluster=localnet" | tee -a "$TX_LOG"
echo "rpc_url=$RPC_URL" | tee -a "$TX_LOG"
echo "sss_token=$(solana address -k target/deploy/sss_token-keypair.json)" | tee -a "$TX_LOG"
echo "sss_transfer_hook=$(solana address -k target/deploy/sss_transfer_hook-keypair.json)" | tee -a "$TX_LOG"

echo "==> Init SSS-1"
OUT_INIT_1="$(run_cli_retry 5 init --preset sss-1)"
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
OUT_INIT_2="$(run_cli_retry 5 init --preset sss-2)"
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

OUT_BL_ADD="$(run_cli --mint "$MINT_SSS2" blacklist add "$OPERATOR_ADDRESS" --reason "localnet-smoke")"
echo "$OUT_BL_ADD" | tee "$ARTIFACT_DIR/sss2-blacklist-add.log"
record_tx "sss2_blacklist_add_tx" "$OUT_BL_ADD"

OUT_SEIZE="$(run_cli --mint "$MINT_SSS2" seize "$OPERATOR_ADDRESS" --to "$TREASURY_ADDRESS")"
echo "$OUT_SEIZE" | tee "$ARTIFACT_DIR/sss2-seize.log"
record_tx "sss2_seize_tx" "$OUT_SEIZE"

OUT_BL_REMOVE="$(run_cli --mint "$MINT_SSS2" blacklist remove "$OPERATOR_ADDRESS")"
echo "$OUT_BL_REMOVE" | tee "$ARTIFACT_DIR/sss2-blacklist-remove.log"
record_tx "sss2_blacklist_remove_tx" "$OUT_BL_REMOVE"

echo "Localnet evidence saved in $ARTIFACT_DIR"
echo "Copy transactions from $TX_LOG into docs/DEPLOYMENT.md localnet section"
