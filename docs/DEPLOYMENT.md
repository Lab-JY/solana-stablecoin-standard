# Devnet Deployment Evidence

This file tracks reproducible deployment evidence required by the Superteam Stablecoin Standard bounty.

## Current Execution Status (as of February 26, 2026 UTC)

- `scripts/localnet-evidence.sh` is implemented for reproducible localnet smoke testing.
- Full localnet smoke flow now passes, including SSS-2 `seize` and blacklist removal.
- Devnet deployment + smoke evidence is complete (program IDs + transaction signatures recorded below).

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

## Devnet Verified Evidence

- Verification date (UTC): `2026-02-26`
- Artifact directories:
  - `docs/deployment-artifacts/20260226-133410-devnet-manual` (transfer-hook deploy)
  - `docs/deployment-artifacts/20260226-134621-devnet-manual` (sss_token deploy)
  - `docs/deployment-artifacts/20260226-134737-smoke` (preset smoke flow)
- Operator: `8bk6EZDQudBjSoFh53FdPiF9qG79n1WggP8hwgyXmEvc`
- Treasury: `BeXg8VcM2MLoPzgEausyH9993eWyBQVua94CuRm7afWo`
- Cluster: `devnet`
- RPC URL: `https://api.devnet.solana.com`
- Commit SHA: `725af93`

## Devnet Program IDs

- `sss_token`: `AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE`
- `sss_transfer_hook`: `Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU`

## Devnet Deploy Transaction Signatures

| Deploy action | Signature |
|---|---|
| sss_transfer_hook deploy | `5YcFpj1MMJzmanPS3VVyNEyFfNvxPFmE5nSPRSxevaK8AeDHEdndyupajJzxLUF5Ro3oPR2zdj1RZjUdCo9UxEDw` |
| sss_token deploy | `4XbfpWvCadd7oRT8T4og9ZLfRmoeBSxE92PYrZW34k4YUDF9E4MfcwqKykxnKTLeZabawtjHgnM1s7Q45Vuw4cyH` |

## Devnet Smoke Transaction Signatures

| Flow (devnet) | Signature |
|---|---|
| sss1_init_tx | `sCDpxiTtecGD9YhWZXCpAcdgkdkARgMQUMT4V4ZEPFCdtUrStEhTdn5oLpV7sUAvdBfh4R3xX4HpigajwHEWD3H` |
| sss1_add_minter_tx | `1qRzAYw6CKsCyzSNtKtkAkVS6rgt6bjBkt41WdRRVmax7jtPwqbtwKrviuCbBAMUt5M7nuTctafe4Lp5kvrCZgE` |
| sss1_mint_tx | `3jNXePnN5sxfYNY4de5HThYvqS5M9fFxaCS3YPe5yA3FJ9AbPKyC9wqaMDDHqwjDsCK65iVBVmyFnAyeCUPdVjGT` |
| sss2_init_tx | `5QsTAir2tzct1xbmFMod7CAzJzueEyFTKgfqNUqcptB84VUq1qMhjNBXP3eEabY6dbh8MiEHjHebu1d971HZDFrL` |
| sss2_add_minter_tx | `2gZHNrUPUx8GJQZtQKytDYA3F2zHLPNrpL2mdKHRT8W7jpdPLrP1YFkqcPhDnV4Febm5WiqCJwdyfqaR3LKCY4PL` |
| sss2_mint_operator_tx | `29o3HBuSyxsHmnTd4UrATuQrjAZuzwG8VSQRiSNQ2TZCzLqSrHwCZXsXWPJpsFi9TgZpmyfrtkg7ibxJGJeL1M1f` |
| sss2_mint_treasury_tx | `434QLp27zrmCck71dmFCb5aS67aasT3rtPEqmVNqCYwq6oFzDdGAUVAuCS9bjDyMUS9k8BoCUp1Pv9UKyLMKBvMT` |
| sss2_blacklist_add_tx | `3Ka81Qxbcu1hCZ5JzEb9khebT1HiiuYrWm76RCeUfgr8LceeUrnyHbB3m7c5tQYL1J4QvBzwgKdnPunSFCbMv6N9` |
| sss2_seize_tx | `579SkXKuKetBhhFRiYx67rx3SkdNK2ZQCYqjxVRwTetiC6dWkR6iK2v94fcMKNXFgCbeBHjkoqULwmCUZdj5zrfW` |
| sss2_blacklist_remove_tx | `43YKw9mvB99u5YXTG38xVEiy8KCeVtoMjXB2NMTGMZRA2KXuW7uDtpZm3MkRSMFZ8cNgUcBGK8qT7qiUa1jUzhJp` |

## Notes

- Script artifacts are saved under `docs/deployment-artifacts/<timestamp>/`.
- Localnet signatures above were copied from `docs/deployment-artifacts/20260226-115508-localnet/transactions.txt`.
