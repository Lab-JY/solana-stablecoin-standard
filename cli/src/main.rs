use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod output;

use commands::{
    audit_log, blacklist, burn, freeze, holders, init, mint, minters, pause, seize, status, thaw,
};

/// SSS Token CLI — Manage Solana Stablecoin Standard tokens
#[derive(Parser)]
#[command(name = "sss-token", version, about)]
struct Cli {
    /// Path to keypair file (default: ~/.config/solana/id.json)
    #[arg(long, global = true)]
    keypair: Option<String>,

    /// Solana RPC URL (default: http://127.0.0.1:8899)
    #[arg(long, global = true)]
    url: Option<String>,

    /// Mint address of the stablecoin (required for most commands)
    #[arg(long, global = true)]
    mint: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new stablecoin
    Init {
        /// Use a preset configuration
        #[arg(long, value_parser = ["sss-1", "sss-2"], group = "config_source")]
        preset: Option<String>,

        /// Path to a custom configuration TOML file
        #[arg(long, group = "config_source")]
        custom: Option<String>,
    },

    /// Mint tokens to a recipient
    Mint {
        /// Recipient wallet address
        recipient: String,

        /// Amount of tokens to mint (in base units)
        amount: u64,
    },

    /// Burn tokens
    Burn {
        /// Amount of tokens to burn (in base units)
        amount: u64,
    },

    /// Freeze a token account
    Freeze {
        /// Address to freeze
        address: String,
    },

    /// Thaw a frozen token account
    Thaw {
        /// Address to thaw
        address: String,
    },

    /// Pause all stablecoin operations
    Pause,

    /// Unpause stablecoin operations
    Unpause,

    /// Show stablecoin configuration and status
    Status,

    /// Show current supply information
    Supply,

    /// Manage the blacklist (SSS-2 only)
    Blacklist {
        #[command(subcommand)]
        action: BlacklistAction,
    },

    /// Seize tokens from an address (SSS-2 only)
    Seize {
        /// Address to seize tokens from
        address: String,

        /// Treasury address to send seized tokens to
        #[arg(long)]
        to: String,
    },

    /// Manage minters
    Minters {
        #[command(subcommand)]
        action: MintersAction,
    },

    /// List token holders
    Holders {
        /// Filter holders with at least this balance
        #[arg(long)]
        min_balance: Option<u64>,
    },

    /// View audit log
    AuditLog {
        /// Filter by action type
        #[arg(long)]
        action: Option<String>,
    },
}

#[derive(Subcommand)]
enum BlacklistAction {
    /// Add an address to the blacklist
    Add {
        /// Address to blacklist
        address: String,

        /// Reason for blacklisting
        #[arg(long)]
        reason: Option<String>,
    },

    /// Remove an address from the blacklist
    Remove {
        /// Address to remove from blacklist
        address: String,
    },
}

#[derive(Subcommand)]
enum MintersAction {
    /// List all current minters
    List,

    /// Add a new minter
    Add {
        /// Minter wallet address
        address: String,

        /// Minting quota (max amount this minter can mint)
        #[arg(long, default_value = "1000000000")]
        quota: u64,
    },

    /// Remove a minter
    Remove {
        /// Minter wallet address to remove
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let cfg = config::CliConfig::load(cli.keypair.as_deref(), cli.url.as_deref())?;

    match cli.command {
        Commands::Init { preset, custom } => {
            init::execute(&cfg, preset.as_deref(), custom.as_deref()).await
        }
        Commands::Mint { recipient, amount } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            mint::execute(&cfg, &mint_addr, &recipient, amount).await
        }
        Commands::Burn { amount } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            burn::execute(&cfg, &mint_addr, amount).await
        }
        Commands::Freeze { address } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            freeze::execute(&cfg, &mint_addr, &address).await
        }
        Commands::Thaw { address } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            thaw::execute(&cfg, &mint_addr, &address).await
        }
        Commands::Pause => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            pause::execute_pause(&cfg, &mint_addr).await
        }
        Commands::Unpause => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            pause::execute_unpause(&cfg, &mint_addr).await
        }
        Commands::Status => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            status::execute_status(&cfg, &mint_addr).await
        }
        Commands::Supply => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            status::execute_supply(&cfg, &mint_addr).await
        }
        Commands::Blacklist { action } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            match action {
                BlacklistAction::Add { address, reason } => {
                    blacklist::execute_add(&cfg, &mint_addr, &address, reason.as_deref()).await
                }
                BlacklistAction::Remove { address } => {
                    blacklist::execute_remove(&cfg, &mint_addr, &address).await
                }
            }
        }
        Commands::Seize { address, to } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            seize::execute(&cfg, &mint_addr, &address, &to).await
        }
        Commands::Minters { action } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            match action {
                MintersAction::List => minters::execute_list(&cfg, &mint_addr).await,
                MintersAction::Add { address, quota } => {
                    minters::execute_add(&cfg, &mint_addr, &address, quota).await
                }
                MintersAction::Remove { address } => {
                    minters::execute_remove(&cfg, &mint_addr, &address).await
                }
            }
        }
        Commands::Holders { min_balance } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            holders::execute(&cfg, &mint_addr, min_balance).await
        }
        Commands::AuditLog { action } => {
            let mint_addr = require_mint(cli.mint.as_deref())?;
            audit_log::execute(&cfg, &mint_addr, action.as_deref()).await
        }
    }
}

fn require_mint(mint: Option<&str>) -> Result<String> {
    mint.map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("--mint <ADDRESS> is required for this command"))
}
