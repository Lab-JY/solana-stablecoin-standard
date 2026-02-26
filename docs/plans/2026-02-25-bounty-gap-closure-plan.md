# SSS Bounty Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close the remaining functional and verification gaps so the repository fully satisfies the Superteam Stablecoin Standard bounty requirements.

**Architecture:** Keep the current three-layer design (on-chain programs + SDK/CLI + backend services), but harden integration boundaries where clients manually encode Anchor instructions. Convert fragile ABI assumptions into tested code paths and evidence-backed deliverables.

**Tech Stack:** Anchor 0.31.x, Token-2022, Rust (cli/services), TypeScript SDK, Mocha/ts-mocha, Trident fuzzing, Docker, Solana Devnet

---

### Task 1: Requirements Traceability Baseline

**Files:**
- Create: `docs/REQUIREMENTS_TRACEABILITY.md`
- Modify: `README.md`

**Step 1: Build a requirement checklist**

List all must-have items from the bounty spec as IDs (SDK core, SSS-1, SSS-2, CLI, services, docs, tests, devnet proof, Docker, bonus items).

**Step 2: Map each ID to code**

For each requirement ID, add implementation references (file path + command/test evidence target).

**Step 3: Mark current status**

Use status values `Done`, `Partial`, `Missing` with short rationale.

**Step 4: Link from README**

Add a short “Bounty Traceability” section in `README.md` linking to `docs/REQUIREMENTS_TRACEABILITY.md`.

**Step 5: Commit**

```bash
git add docs/REQUIREMENTS_TRACEABILITY.md README.md
git commit -m "docs: add bounty requirements traceability matrix"
```

---

### Task 2: Fix CLI ↔ Program ABI Compatibility (Critical)

**Files:**
- Modify: `cli/src/commands/init.rs`
- Modify: `cli/src/commands/mint.rs`
- Modify: `cli/src/commands/burn.rs`
- Modify: `cli/src/commands/freeze.rs`
- Modify: `cli/src/commands/thaw.rs`
- Modify: `cli/src/commands/pause.rs`
- Modify: `cli/src/commands/minters.rs`
- Modify: `cli/src/commands/blacklist.rs`
- Modify: `cli/src/commands/seize.rs`
- Add/Modify tests: `cli/tests/abi_encoding.rs`

**Step 1: Write failing ABI tests**

Add tests that verify instruction discriminators, arg encoding, and account ordering match `programs/sss-token/src/lib.rs` + instruction account structs.

**Step 2: Run tests and verify RED**

Run:
```bash
cargo test -p sss-token-cli abi_encoding -- --nocapture
```
Expected: failures on current manual encodings (especially mint/burn/blacklist/seize).

**Step 3: Implement minimal fixes**

Correct discriminators (`mint_tokens`, `burn_tokens`) and normalize account order/arguments to Anchor expectations for all commands.

**Step 4: Re-run tests and verify GREEN**

Run:
```bash
cargo test -p sss-token-cli abi_encoding -- --nocapture
```
Expected: pass.

**Step 5: Commit**

```bash
git add cli/src/commands cli/tests/abi_encoding.rs
git commit -m "fix(cli): align instruction encoding and account order with on-chain ABI"
```

---

### Task 3: Fix Backend Service Program Invocation Paths

**Files:**
- Modify: `services/mint-burn/src/handlers.rs`
- Modify: `services/compliance/src/handlers.rs`
- Add tests: `services/mint-burn/tests/program_calls.rs`
- Add tests: `services/compliance/tests/program_calls.rs`

**Step 1: Write failing tests for discriminators/account vectors**

Cover mint/burn/compliance instruction builders using deterministic fixtures.

**Step 2: Verify RED**

Run:
```bash
cargo test -p sss-mint-burn
cargo test -p sss-compliance
```
Expected: current ABI mismatches fail.

**Step 3: Patch handlers**

Fix discriminator names and account order; ensure request-to-instruction mapping uses on-chain names and required accounts.

**Step 4: Verify GREEN**

Re-run service tests and ensure all new tests pass.

**Step 5: Commit**

```bash
git add services/mint-burn services/compliance
git commit -m "fix(services): correct Anchor instruction encoding for mint/burn/compliance calls"
```

---

### Task 4: SDK Behavior Coverage Expansion

**Files:**
- Modify: `sdk/core/src/stablecoin.ts`
- Modify: `sdk/core/src/compliance.ts`
- Add tests: `sdk/core/tests/stablecoin.test.ts`
- Add tests: `sdk/core/tests/compliance.test.ts`

**Step 1: Write failing SDK tests**

Add tests for method-to-instruction mapping, preset init params, and compliance module method wiring.

**Step 2: Verify RED**

Run:
```bash
yarn workspace @stbr/sss-token test
```
Expected: failures for current mismatches.

**Step 3: Implement minimal SDK fixes**

Update SDK method calls/args/accounts to exactly match on-chain IDL naming and account expectations.

**Step 4: Verify GREEN**

Run SDK test suite and ensure all tests pass.

**Step 5: Commit**

```bash
git add sdk/core/src sdk/core/tests
git commit -m "test+fix(sdk): validate and align instruction wiring for stablecoin and compliance modules"
```

---

### Task 5: Add Missing Bounty Test Deliverables

**Files:**
- Create: `trident-tests/README.md`
- Create: `trident-tests/fuzz_tests/fuzz_0/test_fuzz.rs`
- Create: `trident-tests/fuzz_tests/fuzz_0/Cargo.toml`
- Modify: `README.md`

**Step 1: Write fuzz scope**

Define fuzz invariants: paused blocks mint/burn, blacklisted cannot transfer (SSS-2), quotas never go negative, total supply conservation with mint/burn.

**Step 2: Implement Trident scaffold**

Add minimal runnable Trident project with one fuzz target.

**Step 3: Execute fuzz job**

Run:
```bash
trident fuzz run-hfuzz fuzz_0
```
Expected: runner starts and produces corpus/artifacts.

**Step 4: Document execution**

Add command and expected output artifacts in `trident-tests/README.md` and reference from `README.md`.

**Step 5: Commit**

```bash
git add trident-tests README.md
git commit -m "test: add trident fuzz testing scaffold and invariants"
```

---

### Task 6: Devnet Deployment Proof & Scripts

**Files:**
- Create: `scripts/deploy-devnet.sh`
- Create: `scripts/devnet-smoke.sh`
- Create: `docs/DEPLOYMENT.md`
- Modify: `README.md`

**Step 1: Write deployment script**

Script should build, deploy both programs to devnet, capture program IDs, and fail fast on errors.

**Step 2: Write smoke script**

Run SSS-1 and SSS-2 lifecycle commands (init, mint, transfer/freeze, blacklist/seize) and emit transaction signatures.

**Step 3: Execute scripts**

Run:
```bash
./scripts/deploy-devnet.sh
./scripts/devnet-smoke.sh
```
Expected: non-zero on failures, deterministic log output on success.

**Step 4: Record evidence**

Capture exact Program IDs, tx signatures, dates, cluster URL, and command transcript summary in `docs/DEPLOYMENT.md`.

**Step 5: Commit**

```bash
git add scripts docs/DEPLOYMENT.md README.md
git commit -m "docs+ops: add devnet deployment evidence and reproducible scripts"
```

---

### Task 7: Final Verification Gate

**Files:**
- Modify (if needed): `README.md`, `docs/REQUIREMENTS_TRACEABILITY.md`

**Step 1: Run Rust validation**

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

**Step 2: Run TS/Anchor validation**

```bash
yarn install
yarn lint
yarn workspace @stbr/sss-token test
anchor test
```

**Step 3: Run docker verification**

```bash
docker compose -f services/docker-compose.yml up --build --abort-on-container-exit
```

**Step 4: Update traceability status**

Mark each requirement ID with concrete command/test/proof evidence.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: final verification and traceability update for bounty submission"
```
