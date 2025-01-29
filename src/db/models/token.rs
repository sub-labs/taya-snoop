use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u64,
    pub total_supply: u64,
    pub trade_volume: f64,
    pub trade_volume_usd: f64,
    pub untracked_volume_usd: f64,
    pub tx_count: u64,
    pub total_liquidity: f64,
    pub derived_eth: f64,
    // TODO: find a way to make the relationship
    // pub token_day_data: Vec<String>,
    // pub pair_day_data_base: Vec<String>,
    // pub pair_day_data_quote: Vec<String>,
    // pub pair_base: Vec<String>,
    // pub pair_quote: Vec<String>,
}
