# Superteam Bounty Requirements Traceability

This matrix maps the Superteam "Build the Solana Stablecoin Standard" requirements to the current implementation and evidence in this repository.

Status legend:
- `Done`: implemented in code/docs
- `Partial`: implemented but needs end-to-end verification or hardening
- `Missing`: not yet implemented/proven in repository

| ID | Requirement | Status | Evidence |
|---|---|---|---|
| BR-01 | Base SDK with configurable extensions/presets | Done | `sdk/core/src/stablecoin.ts`, `sdk/core/src/presets.ts` |
| BR-02 | SSS-1 + SSS-2 standards in one configurable on-chain program | Done | `programs/sss-token/src/lib.rs`, `programs/sss-token/src/instructions/initialize.rs` |
| BR-03 | Core instructions (init, mint, burn, freeze/thaw, pause/unpause, role/authority mgmt) | Done | `programs/sss-token/src/instructions/*.rs` |
| BR-04 | SSS-2 instructions (blacklist add/remove, seize) + graceful gating when disabled | Done | `programs/sss-token/src/instructions/add_to_blacklist.rs`, `programs/sss-token/src/instructions/remove_from_blacklist.rs`, `programs/sss-token/src/instructions/seize.rs` |
| BR-05 | Separate transfer hook blacklist enforcement program | Done | `programs/sss-transfer-hook/src/lib.rs`, `programs/sss-transfer-hook/src/instructions/transfer_hook.rs` |
| BR-06 | Role separation incl. minter quotas | Done | `programs/sss-token/src/state.rs`, `programs/sss-token/src/instructions/update_minter.rs`, `programs/sss-token/src/instructions/update_roles.rs` |
| BR-07 | Admin CLI for presets + operations + compliance + management commands | Partial | `cli/src/main.rs`, `cli/src/commands/*.rs`, `scripts/localnet-evidence.sh`, `docs/deployment-artifacts/20260226-115508-localnet/transactions.txt` (preset + compliance lifecycle verified on localnet; remaining management commands still need dedicated coverage evidence) |
| BR-08 | TypeScript SDK usage for preset/custom init + operations + compliance module | Partial | `sdk/core/src/stablecoin.ts`, `sdk/core/src/compliance.ts` (method wiring aligned; runtime verification pending) |
| BR-09 | Backend services (mint/burn, indexer, compliance, webhook) | Partial | `services/mint-burn`, `services/indexer`, `services/compliance`, `services/webhook` (builds, but full integration proof pending) |
| BR-10 | Dockerized backend + compose | Done | `services/Dockerfile`, `services/docker-compose.yml` |
| BR-11 | Required docs set (README, ARCHITECTURE, SDK, OPERATIONS, SSS-1/2, COMPLIANCE, API) | Done | `README.md`, `docs/ARCHITECTURE.md`, `docs/SDK.md`, `docs/OPERATIONS.md`, `docs/SSS-1.md`, `docs/SSS-2.md`, `docs/COMPLIANCE.md`, `docs/API.md` |
| BR-12 | Integration tests per preset | Partial | `tests/sss-1.ts`, `tests/sss-2.ts`, `tests/helpers/setup.ts` (needs full execution evidence) |
| BR-13 | Trident fuzz tests | Partial | `trident-tests/fuzz_tests/fuzz_0/Cargo.toml`, `trident-tests/fuzz_tests/fuzz_0/test_fuzz.rs`, `trident-tests/README.md` (offline invariant fuzz scaffold runnable; Trident CLI run pending) |
| BR-14 | Devnet deployment proof (Program IDs + tx signatures) | Partial | `docs/DEPLOYMENT.md`, `scripts/deploy-devnet.sh`, `scripts/devnet-smoke.sh`, `scripts/localnet-evidence.sh` (full localnet evidence captured; devnet IDs/signatures still pending) |
| BR-15 | Bonus: SSS-3 PoC spec | Done | `docs/SSS-3.md` |
| BR-16 | Bonus: Oracle module | Done | `programs/sss-oracle/src/lib.rs`, `sdk/core/src/oracle.ts` |
| BR-17 | Bonus: Admin TUI | Done | `tui/src/main.rs`, `tui/src/ui/*.rs` |
| BR-18 | Bonus: example frontend | Done | `app/src/app/page.tsx`, `app/src/components/*.tsx` |

## Current Priority Gaps (Submission-Blocking)

1. Execute Trident CLI fuzz job in a network-enabled environment (`trident fuzz run-hfuzz fuzz_0`) to complete `BR-13`.
2. Produce devnet deployment proof with program IDs + transaction signatures (`BR-14`).
3. Run and capture full integration test evidence for SDK/backend and remaining CLI management flows (`BR-07`, `BR-08`, `BR-09`, `BR-12`).
