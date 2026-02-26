# Trident Fuzz Tests

This directory contains the fuzzing scaffold for bounty-required invariants.

## Planned invariants

1. `paused == true` blocks `mint_tokens` and `burn_tokens`.
2. SSS-2 transfer hook rejects transfers when sender/receiver is blacklisted.
3. Minter quotas never underflow and cannot be exceeded.
4. Total supply accounting remains consistent across mint/burn/seize flows.

## Files

- `fuzz_tests/fuzz_0/Cargo.toml`
- `fuzz_tests/fuzz_0/test_fuzz.rs`

## Offline execution (no Trident dependency)

```bash
cargo run --manifest-path trident-tests/fuzz_tests/fuzz_0/Cargo.toml
```

This validates core invariants locally with deterministic pseudo-random cases.

## Trident execution (when CLI is available)

```bash
trident fuzz run-hfuzz fuzz_0
```

## Status

- Invariant scaffold implemented and runnable offline.
- Trident CLI execution remains pending in environments where `trident` is available.
