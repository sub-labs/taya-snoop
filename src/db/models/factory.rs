use alloy::sol;
use serde::{Deserialize, Serialize};

sol! {
    event PairCreated(address indexed token0, address indexed token1, address pair, uint);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFactory {
    pub address: String,
    pub pair_count: i32,
    pub total_volume_usd: f64,
    pub total_volume_eth: f64,
    pub untracked_volume_usd: f64,
    pub total_liquidity_usd: f64,
    pub total_liquidity_eth: f64,
    pub tx_count: u64,
}
