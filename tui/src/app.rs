use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::time::{Duration, Instant};

use crate::ui;

/// Active panel in the TUI
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Dashboard,
    Operations,
    Blacklist,
    Minters,
}

impl Panel {
    pub fn next(self) -> Self {
        match self {
            Panel::Dashboard => Panel::Operations,
            Panel::Operations => Panel::Blacklist,
            Panel::Blacklist => Panel::Minters,
            Panel::Minters => Panel::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Panel::Dashboard => Panel::Minters,
            Panel::Operations => Panel::Dashboard,
            Panel::Blacklist => Panel::Operations,
            Panel::Minters => Panel::Blacklist,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Panel::Dashboard => "Dashboard",
            Panel::Operations => "Operations",
            Panel::Blacklist => "Blacklist",
            Panel::Minters => "Minters",
        }
    }
}

/// Minter display info
#[derive(Clone, Debug)]
pub struct MinterDisplay {
    pub address: String,
    pub quota: u64,
    pub minted: u64,
    pub remaining: u64,
}

/// Operation log entry
#[derive(Clone, Debug)]
pub struct OperationEntry {
    pub timestamp: String,
    pub action: String,
    pub details: String,
}

/// Stablecoin dashboard data
#[derive(Clone, Debug, Default)]
pub struct DashboardData {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub total_minted: u64,
    pub total_burned: u64,
    pub paused: bool,
    pub active_minters: usize,
    pub compliance_enabled: bool,
    pub authority: String,
    pub mint: String,
    pub connected: bool,
    pub last_refresh: String,
}

/// Main application state
pub struct App {
    pub rpc_client: RpcClient,
    pub mint: Pubkey,
    pub active_panel: Panel,
    pub should_quit: bool,
    pub dashboard: DashboardData,
    pub minters: Vec<MinterDisplay>,
    pub operations: Vec<OperationEntry>,
    pub blacklist_entries: Vec<String>,
    pub scroll_offset: usize,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(rpc_url: &str, mint_address: &str) -> Result<Self> {
        let mint = Pubkey::from_str(mint_address)?;
        let rpc_client = RpcClient::new_with_timeout(rpc_url.to_string(), Duration::from_secs(10));

        Ok(Self {
            rpc_client,
            mint,
            active_panel: Panel::Dashboard,
            should_quit: false,
            dashboard: DashboardData {
                mint: mint_address.to_string(),
                ..Default::default()
            },
            minters: Vec::new(),
            operations: Vec::new(),
            blacklist_entries: Vec::new(),
            scroll_offset: 0,
            last_refresh: Instant::now() - Duration::from_secs(10), // Force immediate refresh
            refresh_interval: Duration::from_secs(5),
            error_message: None,
        })
    }

    pub async fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        loop {
            // Auto-refresh data
            if self.last_refresh.elapsed() >= self.refresh_interval {
                self.refresh_data();
                self.last_refresh = Instant::now();
            }

            // Draw UI
            terminal.draw(|f| ui::draw(f, self))?;

            // Handle events with timeout for auto-refresh
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code);
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => self.active_panel = self.active_panel.next(),
            KeyCode::BackTab => self.active_panel = self.active_panel.prev(),
            KeyCode::Char('r') => {
                self.refresh_data();
                self.last_refresh = Instant::now();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Char('1') => self.active_panel = Panel::Dashboard,
            KeyCode::Char('2') => self.active_panel = Panel::Operations,
            KeyCode::Char('3') => self.active_panel = Panel::Blacklist,
            KeyCode::Char('4') => self.active_panel = Panel::Minters,
            _ => {}
        }
    }

    fn refresh_data(&mut self) {
        self.error_message = None;

        // Try to fetch stablecoin config account
        let config_pda = self.derive_config_pda();
        match self.rpc_client.get_account(&config_pda) {
            Ok(account) => {
                self.dashboard.connected = true;
                self.parse_config_data(&account.data);
                self.dashboard.last_refresh = chrono_now();
            }
            Err(e) => {
                self.dashboard.connected = false;
                self.error_message = Some(format!("RPC error: {}", e));
                self.dashboard.last_refresh = chrono_now();
                // Populate with demo data for display purposes
                self.populate_demo_data();
            }
        }

        // Try to fetch role config
        let roles_pda = self.derive_roles_pda();
        match self.rpc_client.get_account(&roles_pda) {
            Ok(account) => {
                self.parse_roles_data(&account.data);
            }
            Err(_) => {
                // Keep existing data or demo data
            }
        }
    }

    fn derive_config_pda(&self) -> Pubkey {
        let (pda, _) = Pubkey::find_program_address(
            &[b"stablecoin", self.mint.as_ref()],
            &sss_token_program_id(),
        );
        pda
    }

    fn derive_roles_pda(&self) -> Pubkey {
        let config_pda = self.derive_config_pda();
        let (pda, _) = Pubkey::find_program_address(
            &[b"roles", config_pda.as_ref()],
            &sss_token_program_id(),
        );
        pda
    }

    fn parse_config_data(&mut self, data: &[u8]) {
        // Skip 8-byte Anchor discriminator
        if data.len() < 8 {
            return;
        }
        let data = &data[8..];

        // Parse fields based on StablecoinConfig layout
        if data.len() < 32 + 32 {
            return;
        }

        let authority = Pubkey::try_from(&data[0..32]).unwrap_or_default();
        self.dashboard.authority = truncate_pubkey(&authority.to_string());

        // Skip mint (32 bytes) - we already know it
        let mut offset = 64;

        // Parse name (4-byte len + data)
        if let Some((name, new_offset)) = parse_string(data, offset) {
            self.dashboard.name = name;
            offset = new_offset;
        } else {
            return;
        }

        // Parse symbol
        if let Some((symbol, new_offset)) = parse_string(data, offset) {
            self.dashboard.symbol = symbol;
            offset = new_offset;
        } else {
            return;
        }

        // Parse uri (skip it)
        if let Some((_, new_offset)) = parse_string(data, offset) {
            offset = new_offset;
        } else {
            return;
        }

        // decimals (1), paused (1), total_minted (8), total_burned (8)
        if offset + 18 > data.len() {
            return;
        }

        self.dashboard.decimals = data[offset];
        offset += 1;

        self.dashboard.paused = data[offset] != 0;
        offset += 1;

        self.dashboard.total_minted =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or_default());
        offset += 8;

        self.dashboard.total_burned =
            u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or_default());
        offset += 8;

        self.dashboard.total_supply = self
            .dashboard
            .total_minted
            .saturating_sub(self.dashboard.total_burned);

        // enable_permanent_delegate (1), enable_transfer_hook (1)
        if offset + 2 <= data.len() {
            let perm_delegate = data[offset] != 0;
            let transfer_hook = data[offset + 1] != 0;
            self.dashboard.compliance_enabled = perm_delegate && transfer_hook;
        }
    }

    fn parse_roles_data(&mut self, data: &[u8]) {
        // Skip 8-byte Anchor discriminator
        if data.len() < 8 {
            return;
        }
        let data = &data[8..];

        // Skip stablecoin (32), master_authority (32), pauser (32)
        let mut offset = 96;

        // Parse minters vec: 4-byte len + MinterInfo entries
        if offset + 4 > data.len() {
            return;
        }

        let minter_count =
            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or_default()) as usize;
        offset += 4;

        self.minters.clear();
        for _ in 0..minter_count {
            if offset + 48 > data.len() {
                break;
            }
            let address = Pubkey::try_from(&data[offset..offset + 32]).unwrap_or_default();
            offset += 32;
            let quota =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or_default());
            offset += 8;
            let minted =
                u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap_or_default());
            offset += 8;

            self.minters.push(MinterDisplay {
                address: truncate_pubkey(&address.to_string()),
                quota,
                minted,
                remaining: quota.saturating_sub(minted),
            });
        }

        self.dashboard.active_minters = self.minters.len();
    }

    fn populate_demo_data(&mut self) {
        if self.dashboard.name.is_empty() {
            self.dashboard.name = "(not connected)".to_string();
            self.dashboard.symbol = "---".to_string();
            self.dashboard.decimals = 6;

            self.operations = vec![
                OperationEntry {
                    timestamp: "waiting...".to_string(),
                    action: "Connect".to_string(),
                    details: "Waiting for RPC connection".to_string(),
                },
            ];
        }
    }
}

fn sss_token_program_id() -> Pubkey {
    Pubkey::from_str("AhZamuppxULmpM9QGXcZJ9ZR3fvQbDd4gPsxLtDoMQmE").unwrap()
}

fn truncate_pubkey(s: &str) -> String {
    if s.len() > 12 {
        format!("{}...{}", &s[..6], &s[s.len() - 4..])
    } else {
        s.to_string()
    }
}

fn parse_string(data: &[u8], offset: usize) -> Option<(String, usize)> {
    if offset + 4 > data.len() {
        return None;
    }
    let len = u32::from_le_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
    let start = offset + 4;
    if start + len > data.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&data[start..start + len]).to_string();
    Some((s, start + len))
}

fn chrono_now() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let hours = (now % 86400) / 3600;
    let minutes = (now % 3600) / 60;
    let seconds = now % 60;
    format!("{:02}:{:02}:{:02} UTC", hours, minutes, seconds)
}
