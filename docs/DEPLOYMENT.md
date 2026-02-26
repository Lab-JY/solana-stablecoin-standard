# Devnet Deployment Evidence

This file tracks reproducible deployment evidence required by the Superteam Stablecoin Standard bounty.

## Current Execution Status (as of February 26, 2026 UTC)

- `scripts/localnet-evidence.sh` is implemented for reproducible localnet smoke testing.
- Full localnet smoke flow now passes, including SSS-2 `seize` and blacklist removal.
- Devnet deployment + smoke evidence is complete (program IDs + transaction signatures recorded below).

## Localnet Verified Evidence

- Verification date (UTC): `2026-02-26`
- Artifact directory: `docs/deployment-artifacts/20260226-141644-localnet`
- Operator: `FFRaxvpnoN1eXvQLgX92qRfur39FwAzjWTymiQh63boV`
- Treasury: `2NJELMMKnVdqdyRqt8AjFBa4sprUfxDwcYjGbSYFCXrf`

## Localnet SDK Runtime Evidence (BR-08)

- Verification date (UTC): `2026-02-26`
- Artifact directory: `docs/deployment-artifacts/20260226-200009-sdk-localnet`
- Scope: preset init (`SSS-1`, `SSS-2`), custom init, SDK mint/supply operations, compliance add/remove, `SSS-1` graceful compliance failure, `load()` path
- Key outputs:
  - Runtime evidence JSON: `docs/deployment-artifacts/20260226-200009-sdk-localnet/sdk-runtime.json`
  - SDK unit tests: `docs/deployment-artifacts/20260226-200009-sdk-localnet/sdk-core-tests.txt`
  - Run summary: `docs/deployment-artifacts/20260226-200009-sdk-localnet/summary.txt`

## Localnet Program IDs

- `sss_token`: `AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE`
- `sss_transfer_hook`: `Gf5xP5YMRdhb7jRGiDsZW2guwwRMi4RQt4b5r44VPhTU`

## Localnet Smoke Transaction Signatures

| Flow (localnet) | Signature |
|---|---|
| sss1_init_tx | `5o5A2iHSLP4fpvM4w8nx7DM1JXPMGBXAUNZYGW8c6gkxqbU3ha5FsPpRRgigGHEcKfnBg9g1tXUSx3hYUdKGSpXG` |
| sss1_add_minter_tx | `2Qaww6mH3RmBdLAwxrgBNbAoEH7vZ8GbYpSDAvW5QaCrz7QiTQpcTMGG8QEmuUKniKxJngTLAnbYkHA1R62Cz45x` |
| sss1_mint_tx | `46oHZ69EoZtKz3Q1H73FMHjdZbHu3qH6SWhLbpyHQ25bdgcMs6ta4moV7ZvSmZ8YixXEF2dztpxuggnnc723rAsF` |
| sss2_init_tx | `2e87WHRVrDHm7yzQK1XvjL9r1eTtRx5MM5iPwVL24KvxUKAPs1VnzZemRkbnaJF7De5JN2NRju6yBCDRqKrNm6qi` |
| sss2_add_minter_tx | `r6fVUo1VK6m7dqpRuWH2fxbLGiDnJGyBamdXRS5Sf7vVnatbziiRCJ3cxBDG734mjiJeQxsPqYL6ppUoVFo17M8` |
| sss2_mint_operator_tx | `qxnTHybpvLHXBjFfY7X5Fff3faq5TGyQJBKbw5r19uMNSmmF8MEePN32aSfxssucmP4QpudtgWdF5i7gYaE5azk` |
| sss2_mint_treasury_tx | `3TnBYyMSyf2Z54xMFk9pVcK7vTZwaLs15p5vFVSDNQYGX1v6NvCUqAYLoqU327hPzNE4wyBfead9u6uDTwDm4zpZ` |
| sss2_blacklist_add_tx | `5ZMo4HevJkPNti97KbfkRAJTr7PuvMkF6QPaybKkPo2sEnY6NvU2YnpwqpfuvRn4ZQRu2wCmWPMpEq7sEYud3spG` |
| sss2_seize_tx | `2qbcCm75uuJ5faLnUc4SHE9V3hzeq83DVEVqJgpnNP2AMApuBf7dKD7Y8wzqkKnQkU1PneRVKEXNCp3vjyhzgPUg` |
| sss2_blacklist_remove_tx | `2VVZWvwmMCiKH9aCG1bphnwr7nAFkaU6Y8iYKqxFfURYrZ2XdvhYP9m7jzsk39psx2DqFyrNHMT77m75AasiL4mV` |

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
- Localnet signatures above were copied from `docs/deployment-artifacts/20260226-141644-localnet/transactions.txt`.

## Reproduce Evidence

```bash
# Devnet deploy + smoke (network calls can use proxy)
export https_proxy=http://127.0.0.1:7897 http_proxy=http://127.0.0.1:7897 all_proxy=socks5://127.0.0.1:7897
KEYPAIR_PATH=scripts/devnet-keypair.json TREASURY_ADDRESS=BeXg8VcM2MLoPzgEausyH9993eWyBQVua94CuRm7afWo ./scripts/devnet-smoke.sh

# Localnet evidence must run without proxy (script unsets proxy env internally)
./scripts/localnet-evidence.sh

# SDK runtime evidence on localnet (script unsets proxy env internally)
./scripts/sdk-evidence.sh
```
