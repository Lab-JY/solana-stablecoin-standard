# Devnet Deployment Evidence

This file tracks reproducible deployment evidence required by the Superteam Stablecoin Standard bounty.

## Current Execution Status (as of February 26, 2026 UTC)

- `scripts/localnet-evidence.sh` is implemented for reproducible localnet smoke testing.
- Full localnet smoke flow now passes, including SSS-2 `seize` and blacklist removal.
- Devnet evidence is still pending (program IDs + signatures need a successful devnet deployment run).

## Localnet Verified Evidence

- Verification date (UTC): `2026-02-26`
- Artifact directory: `docs/deployment-artifacts/20260226-115508-localnet`
- Operator: `4PETrnUyUvHZaerhhno9kVFievhoWrsTsVKpYKBvQRVs`
- Treasury: `48VGHpCW6ZPXxBicgXzX11BjxheZqcWtLradkVDVKEie`

## Localnet Program IDs

- `sss_token`: `AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE`
- `sss_transfer_hook`: `Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU`

## Localnet Smoke Transaction Signatures

| Flow (localnet) | Signature |
|---|---|
| sss1_init_tx | `2oVoGTqcTPnjmrSbFZwYvqEuVwXQf566kchVkZENcYXczZBcnHxGbgV1CAMNu4kiAsk5bMJqfmr16zuCYAvsaNq7` |
| sss1_add_minter_tx | `5xmZvax7EL2STZ3VHdGwHPE3rqmuHgFAPvynfyekA6XvdcqLEg4yoWnPRWLyL94VBuJ9Zqwb5RFuP9dMmGz8RkQz` |
| sss1_mint_tx | `3S25PtYDf27gdpSmYgJNkgL1LcSojQS5wafeJn7r18WatLPkSJvGeLork6cM5bEh1YyvKz3a6Y4RSV7rmY1cE6sE` |
| sss2_init_tx | `YkJ2qL4Vvhf8v3s32osr92MaUQhiFPXRS1tQTEngdThohLc3D1MU7mLc3pV3xyeAuHSmy9gyecHGcazoc7CanZX` |
| sss2_add_minter_tx | `2f4zAsqn3PmE8v4tzTHGb4pna9cPk2hCSkCmYYrXSyw4gYyWHQrbAHe2AA4gRiuJ2ZwmDZcdZL1e6hZsbcsgSKAW` |
| sss2_mint_operator_tx | `5xPA2ivQUYzw2jYymC1SN8HuhxxTLnJZP8AhkVn2bEaunSkFvRDkuzMmxzA4v7knNjWSDPCGVM7fKvdbYPq7f37K` |
| sss2_mint_treasury_tx | `QHiydUAht9asB2ud9aww48d3fDXAMz6fPNkw7vupKoDR3moYsbEt4z48FSHeVpqYDH6nz1X5iGJzVyr9mKQJEgA` |
| sss2_blacklist_add_tx | `23wtgHv8VZbNWh2F7WLFoTKWqSLZrgjprgFMLNfky3TS9hptiHRwcQwUkLSVrCfS9kKk392xyyRMPJZuTsfhfSX8` |
| sss2_seize_tx | `3avSXEvaMFK9FcdFbThVP4YTgsBJQ2aNPstQ9Ku3VZCnx3w3yciY2r1H1zMHUgpoCA3MZc7G3HhTg5TLb7LGs7qo` |
| sss2_blacklist_remove_tx | `5gR6f9HbBRxnRbzHP3yGxrJBpJ2vstuRKdnJUKnEJhBTPZyEuvqfchajbuPSFVt5akWjxBivXs1BCYtri5JdFyGo` |

## Devnet Evidence (Pending)

- Date (UTC): `TBD`
- Cluster: `devnet`
- RPC URL: `https://api.devnet.solana.com`
- Commit SHA: `TBD`
- `sss_token`: `TBD`
- `sss_transfer_hook`: `TBD`

Run:

```bash
./scripts/deploy-devnet.sh
TREASURY_ADDRESS=<TREASURY_PUBKEY> ./scripts/devnet-smoke.sh
```

## Notes

- Script artifacts are saved under `docs/deployment-artifacts/<timestamp>/`.
- Localnet signatures above were copied from `docs/deployment-artifacts/20260226-115508-localnet/transactions.txt`.
