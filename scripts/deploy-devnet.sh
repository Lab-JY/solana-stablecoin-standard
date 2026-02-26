#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v anchor >/dev/null 2>&1; then
  echo "anchor CLI is required"
  exit 1
fi

if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI is required"
  exit 1
fi

ARTIFACT_DIR="docs/deployment-artifacts/$(date +%Y%m%d-%H%M%S)"
mkdir -p "$ARTIFACT_DIR"

echo "==> Building programs"
anchor build

echo "==> Deploying programs to devnet"
anchor deploy --provider.cluster devnet | tee "$ARTIFACT_DIR/deploy.log"

echo "==> Recording program IDs"
{
  echo "cluster: devnet"
  echo "deployed_at_utc: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "sss_token: $(solana address -k target/deploy/sss_token-keypair.json)"
  echo "sss_transfer_hook: $(solana address -k target/deploy/sss_transfer_hook-keypair.json)"
} | tee "$ARTIFACT_DIR/program-ids.txt"

echo "Deployment artifacts saved in $ARTIFACT_DIR"
