# Solana Stablecoin Standard — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a modular stablecoin SDK for Solana with SSS-1 (minimal) and SSS-2 (compliant) presets, backed by Token-2022 extensions, with Rust CLI/backend and TypeScript SDK.

**Architecture:** Two Anchor programs (sss-token + sss-transfer-hook), TypeScript SDK with factory pattern, Rust CLI (clap v4), Rust backend services (Axum), plus bonus features (TUI, oracle, frontend, SSS-3 spec).

**Tech Stack:** Anchor 0.31.x, Token-2022, TypeScript, Rust, Axum, clap, ratatui, Next.js, SQLite, Docker

**Reference Design:** `docs/plans/2026-02-25-sss-design.md`

---

## Phase 1: Project Scaffolding & Core Program

### Task 1: Initialize Anchor Workspace

**Files:**
- Create: `Anchor.toml`
- Create: `Cargo.toml` (workspace root)
- Create: `package.json` (workspace root)
- Create: `tsconfig.json`
- Create: `.gitignore`
- Create: `CLAUDE.md`
- Create: `programs/sss-token/Cargo.toml`
- Create: `programs/sss-token/Xargo.toml`
- Create: `programs/sss-token/src/lib.rs`
- Create: `programs/sss-transfer-hook/Cargo.toml`
- Create: `programs/sss-transfer-hook/Xargo.toml`
- Create: `programs/sss-transfer-hook/src/lib.rs`

**Step 1: Initialize Anchor project**

Run: `anchor init solana-stablecoin-standard --no-git` in a temp dir, then copy the generated structure. Or manually create the files:

`Anchor.toml`:
```toml
[features]
resolution = true
skip-lint = false

[programs.localnet]
sss_token = "TokenXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
sss_transfer_hook = "HookXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

[programs.devnet]
sss_token = "TokenXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
sss_transfer_hook = "HookXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

[registry]
url = "https://api.apr.dev"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

Root `Cargo.toml`:
```toml
[workspace]
members = [
    "programs/sss-token",
    "programs/sss-transfer-hook",
    "cli",
    "services/shared",
    "services/mint-burn",
    "services/compliance",
    "services/indexer",
    "services/webhook",
    "tui",
]
resolver = "2"

[profile.release]
overflow-checks = true
lto = "fat"
codegen-units = 1
[profile.release.build-override]
opt-level = 3
incremental = false
codegen-units = 1
```

Root `package.json`:
```json
{
  "name": "solana-stablecoin-standard",
  "private": true,
  "workspaces": ["sdk/core", "app"],
  "scripts": {
    "build:sdk": "yarn workspace @stbr/sss-token build",
    "test:sdk": "yarn workspace @stbr/sss-token test",
    "test:integration": "anchor test",
    "lint": "prettier --check 'sdk/**/*.ts' 'tests/**/*.ts'"
  },
  "devDependencies": {
    "@coral-xyz/anchor": "^0.31.0",
    "@solana/web3.js": "^1.95.0",
    "@solana/spl-token": "^0.4.0",
    "chai": "^4.3.10",
    "mocha": "^10.2.0",
    "ts-mocha": "^10.0.0",
    "typescript": "^5.3.0",
    "prettier": "^3.1.0",
    "@types/chai": "^4.3.11",
    "@types/mocha": "^10.0.6",
    "ts-node": "^10.9.2"
  }
}
```

`programs/sss-token/Cargo.toml`:
```toml
[package]
name = "sss-token"
version = "0.1.0"
description = "Solana Stablecoin Standard - Main Program"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "sss_token"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []
idl-build = ["anchor-lang/idl-build", "anchor-spl/idl-build"]

[dependencies]
anchor-lang = { version = "0.31.0", features = ["init-if-needed"] }
anchor-spl = { version = "0.31.0" }
spl-token-2022 = { version = "5.0.2", features = ["no-entrypoint"] }
spl-token-metadata-interface = "0.5.1"
spl-type-length-value = "0.6.0"
```

`programs/sss-transfer-hook/Cargo.toml`:
```toml
[package]
name = "sss-transfer-hook"
version = "0.1.0"
description = "Solana Stablecoin Standard - Transfer Hook Program"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "sss_transfer_hook"

[features]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
cpi = ["no-entrypoint"]
default = []
idl-build = ["anchor-lang/idl-build", "anchor-spl/idl-build"]

[dependencies]
anchor-lang = { version = "0.31.0" }
anchor-spl = { version = "0.31.0" }
spl-token-2022 = { version = "5.0.2", features = ["no-entrypoint"] }
spl-transfer-hook-interface = "0.8.0"
spl-tlv-account-resolution = "0.8.1"
```

Minimal `programs/sss-token/src/lib.rs`:
```rust
use anchor_lang::prelude::*;

declare_id!("TokenXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod sss_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

Minimal `programs/sss-transfer-hook/src/lib.rs`:
```rust
use anchor_lang::prelude::*;

declare_id!("HookXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod sss_transfer_hook {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

**Step 2: Generate program keypairs and update IDs**

Run:
```bash
solana-keygen new -o target/deploy/sss_token-keypair.json --no-bip39-passphrase
solana-keygen new -o target/deploy/sss_transfer_hook-keypair.json --no-bip39-passphrase
```

Extract program IDs and update `declare_id!()` in both lib.rs files and `Anchor.toml`.

**Step 3: Verify build**

Run: `anchor build`
Expected: Both programs compile successfully.

**Step 4: Install Node dependencies**

Run: `yarn install`

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: initialize anchor workspace with sss-token and sss-transfer-hook programs"
```

---

### Task 2: Define State, Errors, Events, Constants for sss-token

**Files:**
- Create: `programs/sss-token/src/state.rs`
- Create: `programs/sss-token/src/error.rs`
- Create: `programs/sss-token/src/events.rs`
- Create: `programs/sss-token/src/constants.rs`
- Modify: `programs/sss-token/src/lib.rs` (add module declarations)

**Step 1: Create `constants.rs`**

```rust
pub const STABLECOIN_SEED: &[u8] = b"stablecoin";
pub const ROLES_SEED: &[u8] = b"roles";
pub const BLACKLIST_SEED: &[u8] = b"blacklist";

pub const MAX_NAME_LEN: usize = 32;
pub const MAX_SYMBOL_LEN: usize = 10;
pub const MAX_URI_LEN: usize = 200;
pub const MAX_REASON_LEN: usize = 128;
pub const MAX_MINTERS: usize = 10;
pub const MAX_BURNERS: usize = 10;
```

**Step 2: Create `error.rs`**

```rust
use anchor_lang::prelude::*;

#[error_code]
pub enum StablecoinError {
    #[msg("The stablecoin is currently paused")]
    Paused,
    #[msg("Unauthorized: caller does not have the required role")]
    Unauthorized,
    #[msg("Minter quota exceeded")]
    MinterQuotaExceeded,
    #[msg("Minter not found")]
    MinterNotFound,
    #[msg("Maximum number of minters reached")]
    MaxMintersReached,
    #[msg("Maximum number of burners reached")]
    MaxBurnersReached,
    #[msg("Compliance module is not enabled for this stablecoin")]
    ComplianceNotEnabled,
    #[msg("Address is already blacklisted")]
    AlreadyBlacklisted,
    #[msg("Address is not blacklisted")]
    NotBlacklisted,
    #[msg("Name exceeds maximum length")]
    NameTooLong,
    #[msg("Symbol exceeds maximum length")]
    SymbolTooLong,
    #[msg("URI exceeds maximum length")]
    UriTooLong,
    #[msg("Reason exceeds maximum length")]
    ReasonTooLong,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Account is frozen")]
    AccountFrozen,
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Minter already exists")]
    MinterAlreadyExists,
    #[msg("Burner already exists")]
    BurnerAlreadyExists,
    #[msg("Burner not found")]
    BurnerNotFound,
}
```

**Step 3: Create `state.rs`**

```rust
use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
pub struct StablecoinConfig {
    /// Master authority
    pub authority: Pubkey,
    /// Token mint address
    pub mint: Pubkey,
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Metadata URI
    pub uri: String,
    /// Token decimals
    pub decimals: u8,
    /// Global pause flag
    pub paused: bool,
    /// Cumulative tokens minted
    pub total_minted: u64,
    /// Cumulative tokens burned
    pub total_burned: u64,
    /// SSS-2: permanent delegate enabled
    pub enable_permanent_delegate: bool,
    /// SSS-2: transfer hook enabled
    pub enable_transfer_hook: bool,
    /// SSS-2: new accounts start frozen
    pub default_account_frozen: bool,
    /// SSS-2: transfer hook program ID
    pub transfer_hook_program: Option<Pubkey>,
    /// PDA bump
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 64],
}

impl StablecoinConfig {
    pub const LEN: usize = 8  // discriminator
        + 32  // authority
        + 32  // mint
        + (4 + MAX_NAME_LEN)    // name (String = 4 byte len + data)
        + (4 + MAX_SYMBOL_LEN)  // symbol
        + (4 + MAX_URI_LEN)     // uri
        + 1   // decimals
        + 1   // paused
        + 8   // total_minted
        + 8   // total_burned
        + 1   // enable_permanent_delegate
        + 1   // enable_transfer_hook
        + 1   // default_account_frozen
        + (1 + 32) // transfer_hook_program (Option<Pubkey>)
        + 1   // bump
        + 64; // _reserved

    pub fn is_compliance_enabled(&self) -> bool {
        self.enable_permanent_delegate && self.enable_transfer_hook
    }
}

#[account]
pub struct RoleConfig {
    /// Parent stablecoin config
    pub stablecoin: Pubkey,
    /// Master authority (same as StablecoinConfig.authority)
    pub master_authority: Pubkey,
    /// Pauser role
    pub pauser: Pubkey,
    /// Minters with quotas
    pub minters: Vec<MinterInfo>,
    /// Burner addresses
    pub burners: Vec<Pubkey>,
    /// SSS-2: blacklister role
    pub blacklister: Pubkey,
    /// SSS-2: seizer role
    pub seizer: Pubkey,
    /// PDA bump
    pub bump: u8,
    /// Reserved for future use
    pub _reserved: [u8; 64],
}

impl RoleConfig {
    pub const LEN: usize = 8  // discriminator
        + 32  // stablecoin
        + 32  // master_authority
        + 32  // pauser
        + (4 + MAX_MINTERS * MinterInfo::LEN) // minters vec
        + (4 + MAX_BURNERS * 32)               // burners vec
        + 32  // blacklister
        + 32  // seizer
        + 1   // bump
        + 64; // _reserved
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct MinterInfo {
    /// Minter public key
    pub address: Pubkey,
    /// Maximum tokens this minter can mint
    pub quota: u64,
    /// Tokens already minted by this minter
    pub minted: u64,
}

impl MinterInfo {
    pub const LEN: usize = 32 + 8 + 8;

    pub fn remaining_quota(&self) -> u64 {
        self.quota.saturating_sub(self.minted)
    }
}

#[account]
pub struct BlacklistEntry {
    /// Parent stablecoin config
    pub stablecoin: Pubkey,
    /// Blacklisted address
    pub address: Pubkey,
    /// Reason for blacklisting
    pub reason: String,
    /// Timestamp when added
    pub added_at: i64,
    /// Who added this entry
    pub added_by: Pubkey,
    /// PDA bump
    pub bump: u8,
}

impl BlacklistEntry {
    pub const LEN: usize = 8  // discriminator
        + 32  // stablecoin
        + 32  // address
        + (4 + MAX_REASON_LEN) // reason
        + 8   // added_at
        + 32  // added_by
        + 1;  // bump
}
```

**Step 4: Create `events.rs`**

```rust
use anchor_lang::prelude::*;

#[event]
pub struct StablecoinInitialized {
    pub mint: Pubkey,
    pub authority: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub preset: String,
}

#[event]
pub struct TokensMinted {
    pub mint: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub minter: Pubkey,
}

#[event]
pub struct TokensBurned {
    pub mint: Pubkey,
    pub amount: u64,
    pub burner: Pubkey,
}

#[event]
pub struct AccountFrozen {
    pub mint: Pubkey,
    pub account: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct AccountThawed {
    pub mint: Pubkey,
    pub account: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct StablecoinPaused {
    pub mint: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct StablecoinUnpaused {
    pub mint: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct MinterUpdated {
    pub mint: Pubkey,
    pub minter: Pubkey,
    pub quota: u64,
    pub action: String,
}

#[event]
pub struct RolesUpdated {
    pub mint: Pubkey,
    pub role: String,
    pub address: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct AuthorityTransferred {
    pub mint: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
pub struct AddressBlacklisted {
    pub mint: Pubkey,
    pub address: Pubkey,
    pub reason: String,
    pub by: Pubkey,
}

#[event]
pub struct AddressUnblacklisted {
    pub mint: Pubkey,
    pub address: Pubkey,
    pub by: Pubkey,
}

#[event]
pub struct TokensSeized {
    pub mint: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64,
    pub by: Pubkey,
}
```

**Step 5: Update `lib.rs` with module declarations**

```rust
use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("REPLACE_WITH_ACTUAL_PROGRAM_ID");

#[program]
pub mod sss_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}
```

**Step 6: Create `programs/sss-token/src/instructions/mod.rs`**

```rust
pub mod initialize;

pub use initialize::*;
```

Create a minimal `programs/sss-token/src/instructions/initialize.rs`:
```rust
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}
```

**Step 7: Verify build**

Run: `anchor build`
Expected: Compiles successfully.

**Step 8: Commit**

```bash
git add programs/sss-token/src/
git commit -m "feat(sss-token): add state, error, events, and constants modules"
```

---

### Task 3: Implement `initialize` Instruction (SSS-1 + SSS-2)

This is the most complex instruction — it creates the Token-2022 mint with the correct extensions based on preset.

**Files:**
- Modify: `programs/sss-token/src/instructions/initialize.rs`
- Modify: `programs/sss-token/src/instructions/mod.rs`
- Modify: `programs/sss-token/src/lib.rs`

**Step 1: Implement initialize instruction**

`programs/sss-token/src/instructions/initialize.rs`:
```rust
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};
use anchor_spl::token_interface::{self, TokenInterface};
use spl_token_2022::{
    extension::{
        metadata_pointer::instruction as metadata_pointer_ix,
        permanent_delegate::instruction as permanent_delegate_ix,
        transfer_hook::instruction as transfer_hook_ix,
        default_account_state::instruction as default_account_state_ix,
        ExtensionType,
    },
    instruction as token_ix,
    state::AccountState,
};
use spl_token_metadata_interface::instruction as metadata_ix;

use crate::constants::*;
use crate::error::StablecoinError;
use crate::events::StablecoinInitialized;
use crate::state::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeParams {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub decimals: u8,
    pub enable_permanent_delegate: bool,
    pub enable_transfer_hook: bool,
    pub default_account_frozen: bool,
}

#[derive(Accounts)]
#[instruction(params: InitializeParams)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: Mint account, initialized via raw invoke_signed
    #[account(mut)]
    pub mint: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = StablecoinConfig::LEN,
        seeds = [STABLECOIN_SEED, mint.key().as_ref()],
        bump,
    )]
    pub stablecoin_config: Account<'info, StablecoinConfig>,

    #[account(
        init,
        payer = authority,
        space = RoleConfig::LEN,
        seeds = [ROLES_SEED, stablecoin_config.key().as_ref()],
        bump,
    )]
    pub role_config: Account<'info, RoleConfig>,

    /// CHECK: Transfer hook program (optional for SSS-2)
    pub transfer_hook_program: Option<UncheckedAccount<'info>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    require!(params.name.len() <= MAX_NAME_LEN, StablecoinError::NameTooLong);
    require!(params.symbol.len() <= MAX_SYMBOL_LEN, StablecoinError::SymbolTooLong);
    require!(params.uri.len() <= MAX_URI_LEN, StablecoinError::UriTooLong);

    let mint = &ctx.accounts.mint;
    let authority = &ctx.accounts.authority;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;

    // Determine extensions to enable
    let mut extension_types = vec![
        ExtensionType::MetadataPointer,
    ];

    if params.enable_permanent_delegate {
        extension_types.push(ExtensionType::PermanentDelegate);
    }

    if params.enable_transfer_hook {
        extension_types.push(ExtensionType::TransferHook);
    }

    if params.default_account_frozen {
        extension_types.push(ExtensionType::DefaultAccountState);
    }

    // Calculate space for mint + extensions + metadata
    let metadata_space = spl_token_2022::extension::ExtensionType::try_calculate_account_len::<
        spl_token_2022::state::Mint,
    >(&extension_types)?;

    // Add space for metadata fields
    let metadata_data_space = pack_metadata_space(&params.name, &params.symbol, &params.uri);
    let total_space = metadata_space + metadata_data_space;
    let lamports = Rent::get()?.minimum_balance(total_space);

    // 1. Create mint account
    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            &mint.key(),
            lamports,
            total_space as u64,
            &token_program.key(),
        ),
        &[
            authority.to_account_info(),
            mint.to_account_info(),
            system_program.to_account_info(),
        ],
        &[],
    )?;

    // 2. Initialize MetadataPointer (self-referencing)
    invoke_signed(
        &metadata_pointer_ix::initialize(
            &token_program.key(),
            &mint.key(),
            Some(authority.key()),
            Some(mint.key()),
        )?,
        &[mint.to_account_info()],
        &[],
    )?;

    // 3. Initialize PermanentDelegate (SSS-2)
    if params.enable_permanent_delegate {
        let stablecoin_config_key = ctx.accounts.stablecoin_config.key();
        invoke_signed(
            &permanent_delegate_ix::initialize(
                &token_program.key(),
                &mint.key(),
                &stablecoin_config_key,
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 4. Initialize TransferHook (SSS-2)
    if params.enable_transfer_hook {
        let hook_program = ctx.accounts.transfer_hook_program
            .as_ref()
            .ok_or(StablecoinError::ComplianceNotEnabled)?;

        invoke_signed(
            &transfer_hook_ix::initialize(
                &token_program.key(),
                &mint.key(),
                Some(authority.key()),
                Some(hook_program.key()),
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 5. Initialize DefaultAccountState (SSS-2 optional)
    if params.default_account_frozen {
        invoke_signed(
            &default_account_state_ix::initialize_default_account_state(
                &token_program.key(),
                &mint.key(),
                &AccountState::Frozen,
            )?,
            &[mint.to_account_info()],
            &[],
        )?;
    }

    // 6. Initialize mint
    let stablecoin_config_key = ctx.accounts.stablecoin_config.key();
    invoke_signed(
        &token_ix::initialize_mint2(
            &token_program.key(),
            &mint.key(),
            &stablecoin_config_key, // mint authority = PDA
            Some(&stablecoin_config_key), // freeze authority = PDA
            params.decimals,
        )?,
        &[mint.to_account_info()],
        &[],
    )?;

    // 7. Initialize token metadata
    invoke_signed(
        &metadata_ix::initialize(
            &token_program.key(),
            &mint.key(),
            authority.key,
            &mint.key(),
            &stablecoin_config_key,
            params.name.clone(),
            params.symbol.clone(),
            params.uri.clone(),
        ),
        &[
            mint.to_account_info(),
            ctx.accounts.stablecoin_config.to_account_info(),
        ],
        &[&[
            STABLECOIN_SEED,
            mint.key().as_ref(),
            &[ctx.bumps.stablecoin_config],
        ]],
    )?;

    // 8. Set stablecoin config state
    let config = &mut ctx.accounts.stablecoin_config;
    config.authority = authority.key();
    config.mint = mint.key();
    config.name = params.name.clone();
    config.symbol = params.symbol.clone();
    config.uri = params.uri.clone();
    config.decimals = params.decimals;
    config.paused = false;
    config.total_minted = 0;
    config.total_burned = 0;
    config.enable_permanent_delegate = params.enable_permanent_delegate;
    config.enable_transfer_hook = params.enable_transfer_hook;
    config.default_account_frozen = params.default_account_frozen;
    config.transfer_hook_program = ctx.accounts.transfer_hook_program
        .as_ref()
        .map(|p| p.key());
    config.bump = ctx.bumps.stablecoin_config;
    config._reserved = [0u8; 64];

    // 9. Set role config state
    let roles = &mut ctx.accounts.role_config;
    roles.stablecoin = config.key();
    roles.master_authority = authority.key();
    roles.pauser = authority.key();
    roles.minters = vec![];
    roles.burners = vec![];
    roles.blacklister = authority.key();
    roles.seizer = authority.key();
    roles.bump = ctx.bumps.role_config;
    roles._reserved = [0u8; 64];

    // 10. Emit event
    let preset = if params.enable_permanent_delegate && params.enable_transfer_hook {
        "SSS-2"
    } else {
        "SSS-1"
    };

    emit!(StablecoinInitialized {
        mint: mint.key(),
        authority: authority.key(),
        name: params.name,
        symbol: params.symbol,
        decimals: params.decimals,
        preset: preset.to_string(),
    });

    Ok(())
}

fn pack_metadata_space(name: &str, symbol: &str, uri: &str) -> usize {
    // TLV metadata: type (2) + length (2) + data for each field
    // Update authority (32) + mint (32) + name + symbol + uri + padding
    let base = 4 + 4 + 32 + 32; // type + length + update_authority + mint
    let name_len = 4 + name.len();
    let symbol_len = 4 + symbol.len();
    let uri_len = 4 + uri.len();
    base + name_len + symbol_len + uri_len + 256 // extra padding for TLV overhead
}
```

**Step 2: Update lib.rs**

```rust
use anchor_lang::prelude::*;

pub mod constants;
pub mod error;
pub mod events;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("REPLACE_WITH_ACTUAL_PROGRAM_ID");

#[program]
pub mod sss_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }
}
```

**Step 3: Verify build**

Run: `anchor build`
Expected: Compiles successfully (may have warnings about unused imports — fine for now).

**Step 4: Commit**

```bash
git add programs/sss-token/
git commit -m "feat(sss-token): implement initialize instruction with Token-2022 extensions"
```

---

### Task 4: Implement Core Instructions (mint, burn, freeze, thaw, pause, unpause)

**Files:**
- Create: `programs/sss-token/src/instructions/mint.rs`
- Create: `programs/sss-token/src/instructions/burn.rs`
- Create: `programs/sss-token/src/instructions/freeze_account.rs`
- Create: `programs/sss-token/src/instructions/thaw_account.rs`
- Create: `programs/sss-token/src/instructions/pause.rs`
- Create: `programs/sss-token/src/instructions/unpause.rs`
- Modify: `programs/sss-token/src/instructions/mod.rs`
- Modify: `programs/sss-token/src/lib.rs`

**Step 1: Implement each instruction file**

Each instruction follows the pattern:
1. Define `#[derive(Accounts)]` struct with Anchor constraints
2. Validate role from RoleConfig
3. Check `!paused` for operational instructions
4. Execute via Token-2022 CPI
5. Update state counters
6. Emit event

Key patterns per instruction:

**mint.rs**: Check minter is in `role_config.minters`, check `remaining_quota() >= amount`, CPI `token_2022::mint_to` using stablecoin_config PDA signer seeds, increment `minter.minted` and `config.total_minted`.

**burn.rs**: Check burner is in `role_config.burners`, CPI `token_2022::burn` with user as authority, increment `config.total_burned`.

**freeze_account.rs**: Check caller is authority or pauser, CPI `token_2022::freeze_account` using stablecoin_config PDA signer seeds.

**thaw_account.rs**: Check caller is authority or pauser, CPI `token_2022::thaw_account` using stablecoin_config PDA signer seeds.

**pause.rs**: Check caller is pauser, set `config.paused = true`.

**unpause.rs**: Check caller is pauser, set `config.paused = false`.

**Step 2: Wire up in mod.rs and lib.rs**

Add all instruction modules to `mod.rs` and corresponding `pub fn` entries in `lib.rs`.

**Step 3: Verify build**

Run: `anchor build`

**Step 4: Commit**

```bash
git add programs/sss-token/
git commit -m "feat(sss-token): implement core instructions (mint, burn, freeze, thaw, pause, unpause)"
```

---

### Task 5: Implement Role Management Instructions

**Files:**
- Create: `programs/sss-token/src/instructions/update_minter.rs`
- Create: `programs/sss-token/src/instructions/update_roles.rs`
- Create: `programs/sss-token/src/instructions/transfer_authority.rs`
- Modify: `programs/sss-token/src/instructions/mod.rs`
- Modify: `programs/sss-token/src/lib.rs`

**Step 1: Implement update_minter**

Supports three actions via enum: `Add { address, quota }`, `Remove { address }`, `UpdateQuota { address, new_quota }`. Authority only.

**Step 2: Implement update_roles**

Allows authority to update `pauser`, `blacklister`, `seizer`, and `burners` list.

**Step 3: Implement transfer_authority**

Two-step: set pending authority, then accept. Prevents accidental lockout.

**Step 4: Verify build, commit**

```bash
git add programs/sss-token/
git commit -m "feat(sss-token): implement role management instructions"
```

---

### Task 6: Implement SSS-2 Compliance Instructions

**Files:**
- Create: `programs/sss-token/src/instructions/add_to_blacklist.rs`
- Create: `programs/sss-token/src/instructions/remove_from_blacklist.rs`
- Create: `programs/sss-token/src/instructions/seize.rs`
- Modify: `programs/sss-token/src/instructions/mod.rs`
- Modify: `programs/sss-token/src/lib.rs`

**Step 1: Implement add_to_blacklist**

Check `config.is_compliance_enabled()`. Check caller is blacklister. Create `BlacklistEntry` PDA at seeds `["blacklist", mint, address]`. Emit `AddressBlacklisted` event.

**Step 2: Implement remove_from_blacklist**

Check compliance enabled. Check caller is blacklister. Close `BlacklistEntry` PDA (return rent). Emit `AddressUnblacklisted`.

**Step 3: Implement seize**

Check compliance enabled. Check caller is seizer. Use permanent delegate authority (stablecoin_config PDA) to CPI `token_2022::transfer_checked` from target to treasury. Emit `TokensSeized`.

Key: The stablecoin_config PDA is the permanent delegate. We sign with `[STABLECOIN_SEED, mint.as_ref(), &[bump]]`.

**Step 4: Verify build, commit**

```bash
git add programs/sss-token/
git commit -m "feat(sss-token): implement SSS-2 compliance instructions (blacklist, seize)"
```

---

### Task 7: Implement Transfer Hook Program

**Files:**
- Modify: `programs/sss-transfer-hook/src/lib.rs`
- Create: `programs/sss-transfer-hook/src/state.rs`
- Create: `programs/sss-transfer-hook/src/error.rs`
- Create: `programs/sss-transfer-hook/src/instructions/mod.rs`
- Create: `programs/sss-transfer-hook/src/instructions/initialize_extra_account_meta.rs`
- Create: `programs/sss-transfer-hook/src/instructions/transfer_hook.rs`

**Step 1: Define state and error**

State: None needed (reads BlacklistEntry from sss-token program via extra accounts).
Error: `BlacklistedSender`, `BlacklistedReceiver`.

**Step 2: Implement initialize_extra_account_meta_list**

Creates the ExtraAccountMetas PDA that tells Token-2022 which additional accounts to pass during transfers. Must register:
- Sender's BlacklistEntry PDA (derived from sss-token program)
- Receiver's BlacklistEntry PDA (derived from sss-token program)

Uses `spl_tlv_account_resolution::account::ExtraAccountMeta` with PDA seeds using dynamic derivation from the source/destination account data (extract owner at byte offset 32, length 32).

**Step 3: Implement transfer_hook**

The actual hook function:
1. Check `transferring` flag on source account (security requirement)
2. Read sender BlacklistEntry PDA — if account has data (not empty/zeroed), sender is blacklisted → error
3. Read receiver BlacklistEntry PDA — if account has data, receiver is blacklisted → error
4. If neither is blacklisted, return Ok (transfer proceeds)

**Step 4: Implement Anchor fallback**

Required because SPL Transfer Hook Interface uses different discriminators than Anchor:

```rust
pub fn fallback<'info>(
    program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    data: &[u8],
) -> Result<()> {
    let instruction = TransferHookInstruction::unpack(data)?;
    match instruction {
        TransferHookInstruction::Execute { amount } => {
            __private::__global::transfer_hook(program_id, accounts, amount)
        }
        _ => Err(ProgramError::InvalidInstructionData.into()),
    }
}
```

**Step 5: Verify build, commit**

```bash
git add programs/sss-transfer-hook/
git commit -m "feat(transfer-hook): implement blacklist enforcement transfer hook program"
```

---

## Phase 2: Integration Tests

### Task 8: SSS-1 Integration Tests

**Files:**
- Create: `tests/helpers/setup.ts` (shared test utilities)
- Create: `tests/sss-1.ts`

**Step 1: Create test helpers**

Utility functions: `createMint22`, `createTokenAccount`, `airdrop`, `deriveStablecoinPDA`, `deriveRolePDA`, PDA helper for all seeds.

**Step 2: Write SSS-1 test suite**

Tests:
1. Initialize SSS-1 stablecoin (no compliance extensions)
2. Add minter with quota
3. Mint tokens to recipient
4. Transfer tokens (standard transfer_checked)
5. Freeze account → verify transfer fails
6. Thaw account → verify transfer succeeds
7. Pause → verify mint fails
8. Unpause → verify mint succeeds
9. Burn tokens
10. Minter quota enforcement (mint up to quota, fail on over-quota)
11. Unauthorized access (non-minter tries to mint → error)
12. SSS-2 instructions fail gracefully (add_to_blacklist → ComplianceNotEnabled)

**Step 3: Run tests**

Run: `anchor test -- --features ""`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add tests/
git commit -m "test(sss-1): add SSS-1 integration tests"
```

---

### Task 9: SSS-2 Integration Tests

**Files:**
- Create: `tests/sss-2.ts`
- Create: `tests/compliance.ts`

**Step 1: Write SSS-2 initialization test**

Initialize with `enable_permanent_delegate: true, enable_transfer_hook: true`. Verify all extensions are set on the mint.

**Step 2: Write compliance test suite**

Tests:
1. Initialize SSS-2 stablecoin
2. Initialize transfer hook extra account metas
3. Mint tokens to user A and user B
4. Transfer from A to B succeeds (neither blacklisted)
5. Blacklist user A
6. Transfer from A to B fails (sender blacklisted)
7. Transfer from B to A fails (receiver blacklisted)
8. Remove A from blacklist → transfer succeeds
9. Seize tokens from user B to treasury
10. Verify seize used permanent delegate (tokens moved without B's signature)
11. Full lifecycle: init → mint → transfer → blacklist → seize → unblacklist

**Step 3: Run tests**

Run: `anchor test`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add tests/
git commit -m "test(sss-2): add SSS-2 compliance integration tests"
```

---

## Phase 3: TypeScript SDK

### Task 10: Scaffold SDK Package

**Files:**
- Create: `sdk/core/package.json`
- Create: `sdk/core/tsconfig.json`
- Create: `sdk/core/src/index.ts`
- Create: `sdk/core/src/types.ts`
- Create: `sdk/core/src/pda.ts`
- Create: `sdk/core/src/errors.ts`
- Create: `sdk/core/src/presets.ts`

**Step 1: Create package.json**

```json
{
  "name": "@stbr/sss-token",
  "version": "0.1.0",
  "description": "Solana Stablecoin Standard SDK",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "files": ["dist"],
  "scripts": {
    "build": "tsc",
    "test": "mocha -r ts-node/register tests/**/*.ts",
    "lint": "prettier --check src/ tests/"
  },
  "dependencies": {
    "@coral-xyz/anchor": "^0.31.0",
    "@solana/web3.js": "^1.95.0",
    "@solana/spl-token": "^0.4.0",
    "bn.js": "^5.2.1"
  },
  "devDependencies": {
    "typescript": "^5.3.0",
    "@types/bn.js": "^5.1.5",
    "mocha": "^10.2.0",
    "@types/mocha": "^10.0.6",
    "chai": "^4.3.10",
    "@types/chai": "^4.3.11",
    "ts-node": "^10.9.2"
  }
}
```

**Step 2: Implement PDA derivation, types, presets, errors**

All derived from the on-chain program's seeds and state.

**Step 3: Commit**

```bash
git add sdk/
git commit -m "feat(sdk): scaffold TypeScript SDK package"
```

---

### Task 11: Implement SolanaStablecoin Class

**Files:**
- Create: `sdk/core/src/stablecoin.ts`
- Create: `sdk/core/src/compliance.ts`
- Modify: `sdk/core/src/index.ts`

**Step 1: Implement SolanaStablecoin**

Factory pattern with `static async create()` and `static async load()`. All methods map to program instructions. Lazy state caching with `refresh()`.

**Step 2: Implement ComplianceModule**

Separate class for SSS-2 operations. SolanaStablecoin returns a real ComplianceModule for SSS-2, or a proxy that throws `ComplianceNotEnabled` for SSS-1.

**Step 3: Write SDK unit tests**

`sdk/core/tests/pda.test.ts`, `sdk/core/tests/presets.test.ts`

**Step 4: Commit**

```bash
git add sdk/
git commit -m "feat(sdk): implement SolanaStablecoin and ComplianceModule classes"
```

---

## Phase 4: Rust CLI

### Task 12: Scaffold and Implement CLI

**Files:**
- Create: `cli/Cargo.toml`
- Create: `cli/src/main.rs`
- Create: `cli/src/commands/mod.rs`
- Create: `cli/src/commands/init.rs`
- Create: `cli/src/commands/mint.rs`
- Create: `cli/src/commands/burn.rs`
- Create: `cli/src/commands/freeze.rs`
- Create: `cli/src/commands/thaw.rs`
- Create: `cli/src/commands/pause.rs`
- Create: `cli/src/commands/status.rs`
- Create: `cli/src/commands/blacklist.rs`
- Create: `cli/src/commands/seize.rs`
- Create: `cli/src/commands/minters.rs`
- Create: `cli/src/commands/holders.rs`
- Create: `cli/src/commands/audit_log.rs`
- Create: `cli/src/config.rs`
- Create: `cli/src/output.rs`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "sss-token-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "sss-token"
path = "src/main.rs"

[dependencies]
clap = { version = "4", features = ["derive"] }
solana-sdk = "2.1"
solana-client = "2.1"
anchor-client = "0.31.0"
spl-token-2022 = "5.0.2"
serde = { version = "1", features = ["derive"] }
toml = "0.8"
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
colored = "2"
```

**Step 2: Implement main.rs with clap derive**

Define all subcommands matching the requirements spec. Each command module calls the on-chain program via `anchor-client` or raw Solana RPC.

**Step 3: Implement config.rs**

Support `--preset sss-1|sss-2` and `--custom config.toml`. Parse TOML/JSON config files into `InitializeParams`.

**Step 4: Verify build**

Run: `cargo build -p sss-token-cli`

**Step 5: Commit**

```bash
git add cli/
git commit -m "feat(cli): implement Rust CLI with all sss-token commands"
```

---

## Phase 5: Backend Services

### Task 13: Shared Library and Mint/Burn Service

**Files:**
- Create: `services/shared/Cargo.toml`
- Create: `services/shared/src/lib.rs` (Solana RPC client, DB, types, logging)
- Create: `services/mint-burn/Cargo.toml`
- Create: `services/mint-burn/src/main.rs`
- Create: `services/mint-burn/src/routes.rs`
- Create: `services/mint-burn/src/handlers.rs`

**Step 1: Implement shared library**

Common types, Solana client wrapper, SQLite connection pool (sqlx), structured logging (tracing + tracing-subscriber), health check handler.

**Step 2: Implement mint-burn service**

Axum routes: `POST /mint`, `POST /burn`, `GET /supply`, `GET /health`.
Each handler: validate request → build transaction → send to Solana → log to DB → respond.

**Step 3: Verify build, commit**

```bash
git add services/
git commit -m "feat(services): implement shared library and mint-burn service"
```

---

### Task 14: Compliance, Indexer, and Webhook Services

**Files:**
- Create: `services/compliance/` (full service)
- Create: `services/indexer/` (full service)
- Create: `services/webhook/` (full service)
- Create: `services/docker-compose.yml`
- Create: `services/Dockerfile` (multi-stage build)

**Step 1: Implement compliance service**

Routes: `POST /blacklist`, `DELETE /blacklist/:address`, `GET /blacklist`, `GET /audit-trail`, `GET /health`.

**Step 2: Implement indexer service**

WebSocket subscription to program logs. Parse events, store in SQLite. Forward to webhook service.

**Step 3: Implement webhook service**

Routes: `POST /webhooks` (register), `DELETE /webhooks/:id`, `GET /webhooks`. Background task: deliver events with exponential backoff retry.

**Step 4: Create Docker setup**

`docker-compose.yml` with all 4 services + shared volume for SQLite.

**Step 5: Verify `docker compose build`, commit**

```bash
git add services/
git commit -m "feat(services): implement compliance, indexer, webhook services with Docker"
```

---

## Phase 6: Fuzz Tests & Devnet Deployment

### Task 15: Trident Fuzz Tests

**Files:**
- Create: `trident-tests/fuzz_tests/` (Trident structure)

**Step 1: Implement fuzz test flows**

Flows: `initialize`, `mint`, `burn`, `freeze`, `thaw`, `blacklist_add`, `seize`, `roundtrip`.
Invariants: `total_minted >= total_burned`, minter quotas never negative, blacklist entries consistent.

**Step 2: Run fuzz tests**

Run: `trident fuzz run-hfuzz fuzz_0`

**Step 3: Commit**

```bash
git add trident-tests/
git commit -m "test: add Trident fuzz tests for sss-token"
```

---

### Task 16: Devnet Deployment & Smoke Tests

**Files:**
- Create: `scripts/deploy-devnet.sh`
- Create: `scripts/smoke-test-sss1.ts`
- Create: `scripts/smoke-test-sss2.ts`

**Step 1: Deploy to devnet**

Build programs, deploy via `anchor deploy --provider.cluster devnet`. Record Program IDs.

**Step 2: Run smoke tests**

Execute full lifecycle on devnet: create SSS-1 and SSS-2 tokens, mint, transfer, freeze, blacklist, seize. Record transaction signatures.

**Step 3: Document deployment proof**

Save Program IDs and example transaction signatures to `docs/DEPLOYMENT.md`.

**Step 4: Commit**

```bash
git add scripts/ docs/DEPLOYMENT.md
git commit -m "test: add devnet deployment scripts and smoke tests"
```

---

## Phase 7: Documentation

### Task 17: Write All Documentation

**Files:**
- Create: `docs/README.md` (or update root `README.md`)
- Create: `docs/ARCHITECTURE.md`
- Create: `docs/SDK.md`
- Create: `docs/OPERATIONS.md`
- Create: `docs/SSS-1.md`
- Create: `docs/SSS-2.md`
- Create: `docs/SSS-3.md`
- Create: `docs/COMPLIANCE.md`
- Create: `docs/API.md`

**Step 1: Write each document**

Follow SVS repo quality standards. Include architecture diagrams (ASCII), code examples, PDA derivation tables, instruction references, error codes.

**Step 2: Write root README.md**

Overview, quick start for both presets, architecture diagram, links to all docs.

**Step 3: Commit**

```bash
git add docs/ README.md
git commit -m "docs: add comprehensive documentation for all standards and SDK"
```

---

## Phase 8: Bonus Features

### Task 18: Oracle Integration Module

**Files:**
- Create: `programs/sss-oracle/` (Switchboard oracle program)
- Create: `sdk/core/src/oracle.ts` (SDK module)

Separate Anchor program that reads Switchboard price feeds for non-USD pegs. Used for mint/redeem pricing only — does not modify the core SSS-1/SSS-2 token.

**Commit:** `feat(bonus): add Switchboard oracle integration module`

---

### Task 19: Admin TUI

**Files:**
- Create: `tui/Cargo.toml`
- Create: `tui/src/main.rs`
- Create: `tui/src/app.rs`
- Create: `tui/src/ui/` (dashboard, blacklist, minters panels)

ratatui-based terminal UI: supply dashboard, recent operations log, blacklist management, minter quota monitoring. Reads data via Solana RPC.

**Commit:** `feat(bonus): add ratatui admin TUI`

---

### Task 20: Frontend Example

**Files:**
- Create: `app/` (Next.js project)

Simple web UI using `@stbr/sss-token` SDK. Pages: create stablecoin, mint/burn, view holders, manage blacklist (SSS-2). Uses wallet adapter for signing.

**Commit:** `feat(bonus): add Next.js frontend example`

---

### Task 21: SSS-3 Private Stablecoin Spec

**Files:**
- Create: `docs/SSS-3.md`

Documentation-only PoC. Describes architecture for confidential transfers + scoped allowlists. Notes that Confidential Transfers are currently disabled on mainnet/devnet. Includes future implementation roadmap.

**Commit:** `docs(bonus): add SSS-3 private stablecoin specification`

---

## Phase 9: Final Polish

### Task 22: CLAUDE.md, Pre-commit Hooks, CI Setup

**Files:**
- Create/Update: `CLAUDE.md`
- Create: `.github/workflows/ci.yml` (if desired)

Set up pre-commit checks: `cargo fmt --check`, `cargo clippy`, `cargo test --lib`, `prettier --check`, `tsc --noEmit`.

**Commit:** `chore: add CLAUDE.md, pre-commit hooks, and CI configuration`

---

### Task 23: Final Review and PR Preparation

**Step 1: Run full test suite**

```bash
anchor build && anchor test
cargo test --workspace
cargo clippy --workspace -- -D warnings
yarn workspace @stbr/sss-token test
docker compose -f services/docker-compose.yml build
```

**Step 2: Verify devnet deployment**

Ensure Program IDs and transactions are documented.

**Step 3: Create PR**

Single PR with full SDK + SSS-1 + SSS-2 + all bonus features.

---

## Task Dependency Graph

```
Task 1 (scaffold) → Task 2 (state/errors) → Task 3 (initialize) → Task 4 (core instructions) → Task 5 (roles) → Task 6 (compliance)
                                                                                                                       ↓
Task 7 (transfer hook) ←────────────────────────────────────────────────────────────────────────────────────────────────┘
                                                                                                                       ↓
Task 8 (SSS-1 tests) + Task 9 (SSS-2 tests) ← requires Task 7
                                                                                                                       ↓
Task 10 (SDK scaffold) + Task 11 (SDK impl) ← can start after Task 6
Task 12 (CLI) ← can start after Task 6
Task 13 + 14 (services) ← can start after Task 6
                                                                                                                       ↓
Task 15 (fuzz) ← after Task 9
Task 16 (devnet) ← after Task 9
Task 17 (docs) ← after Task 11
Task 18-21 (bonus) ← after Task 16
Task 22-23 (polish) ← after all above
```

## Parallelization Opportunities

These task groups can be developed in parallel after Phase 1 completes:

- **Group A**: Integration tests (Tasks 8-9)
- **Group B**: TypeScript SDK (Tasks 10-11)
- **Group C**: Rust CLI (Task 12)
- **Group D**: Backend services (Tasks 13-14)

After integration tests pass:
- **Group E**: Fuzz tests + Devnet (Tasks 15-16)
- **Group F**: Documentation (Task 17)
- **Group G**: Bonus features (Tasks 18-21)
