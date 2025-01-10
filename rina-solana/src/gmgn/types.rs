use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TopHoldersResponse {
    pub code: i32,
    pub msg: String,
    pub data: Vec<HolderInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HolderInfo {
    pub address: String,
    pub addr_type: i32,
    pub amount_cur: f64,
    pub usd_value: f64,
    pub cost_cur: f64,
    pub sell_amount_cur: f64,
    pub sell_amount_percentage: f64,
    pub sell_volume_cur: f64,
    pub buy_volume_cur: f64,
    pub buy_amount_cur: f64,
    pub netflow_usd: f64,
    pub netflow_amount: f64,
    pub buy_tx_count_cur: i32,
    pub sell_tx_count_cur: i32,
    pub wallet_tag_v2: String,
    pub eth_balance: String,
    pub sol_balance: String,
    pub trx_balance: String,
    pub balance: String,
    pub profit: f64,
    pub realized_profit: f64,
    pub unrealized_profit: f64,
    pub profit_change: Option<f64>,
    pub amount_percentage: f64,
    pub avg_cost: Option<f64>,
    pub avg_sold: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub maker_token_tags: Vec<String>,
    pub name: Option<String>,
    pub avatar: Option<String>,
    pub twitter_username: Option<String>,
    pub twitter_name: Option<String>,
    pub tag_rank: HashMap<String, Option<i32>>,
    pub last_active_timestamp: Option<i64>,
    #[serde(default)]
    pub created_at: i64,
    pub accu_amount: f64,
    pub accu_cost: f64,
    pub cost: f64,
    pub total_cost: f64,
    pub transfer_in: bool,
    pub is_new: bool,
    pub native_transfer: NativeTransfer,
    pub is_suspicious: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NativeTransfer {
    pub name: Option<String>,
    pub from_address: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfoResponse {
    pub code: i32,
    pub reason: String,
    pub message: String,
    pub data: TokenInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub logo: String,
    pub biggest_pool_address: String,
    pub open_timestamp: i64,
    pub holder_count: i32,
    pub circulating_supply: String,
    pub total_supply: String,
    pub max_supply: String,
    pub liquidity: String,
    pub creation_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletHoldingsResponse {
    pub code: i32,
    pub reason: String,
    pub message: String,
    pub data: WalletHoldingsData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletHoldingsData {
    pub holdings: Vec<HoldingInfo>,
    pub next: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HoldingInfo {
    pub token: TokenHoldingInfo,
    pub balance: String,
    pub usd_value: String,
    pub realized_profit_30d: String,
    pub realized_profit: String,
    pub realized_pnl: String,
    pub realized_pnl_30d: String,
    pub unrealized_profit: String,
    pub unrealized_pnl: String,
    pub total_profit: String,
    pub total_profit_pnl: String,
    pub avg_cost: String,
    pub avg_sold: String,
    pub buy_30d: i32,
    pub sell_30d: i32,
    pub sells: i32,
    pub price: String,
    pub cost: String,
    pub position_percent: String,
    pub last_active_timestamp: i64,
    pub history_sold_income: String,
    pub history_bought_cost: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenHoldingInfo {
    pub address: String,
    pub token_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub logo: String,
    pub price_change_6h: String,
    pub is_show_alert: bool,
    pub is_honeypot: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRankResponse {
    pub code: i32,
    pub msg: String,
    pub data: SwapRankData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRankData {
    pub rank: Vec<TokenRankInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRankInfo {
    pub id: i64,
    pub chain: String,
    pub address: String,
    pub symbol: String,
    pub price: f64,
    pub price_change_percent: f64,
    pub swaps: f64,
    pub volume: f64,
    pub liquidity: f64,
    pub market_cap: f64,
    pub hot_level: i32,
    pub pool_creation_timestamp: i64,
    pub holder_count: f64,
    pub twitter_username: Option<String>,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub open_timestamp: i64,
    pub price_change_percent1m: f64,
    pub price_change_percent5m: f64,
    pub price_change_percent1h: f64,
    pub buys: f64,
    pub sells: f64,
    pub initial_liquidity: f64,
    pub is_show_alert: bool,
    pub top_10_holder_rate: f64,
    pub renounced_mint: i32,
    pub renounced_freeze_account: i32,
    pub burn_ratio: Option<String>,
    pub burn_status: Option<String>,
    pub launchpad: Option<String>,
    pub dev_token_burn_amount: Option<String>,
    pub dev_token_burn_ratio: Option<f64>,
    pub dexscr_ad: i32,
    pub dexscr_update_link: i32,
    pub cto_flag: i32,
    pub twitter_change_flag: i32,
    pub creator_token_status: Option<String>,
    pub creator_close: Option<bool>,
    pub launchpad_status: i32,
    pub rat_trader_amount_rate: f64,
    pub bluechip_owner_percentage: f64,
    pub smart_degen_count: u32,
    pub renowned_count: f64,
    pub is_wash_trading: bool,
}
