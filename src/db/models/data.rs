use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDexDayData {
    pub id: String,
    pub date: i32,
    pub daily_volume_eth: f64,
    pub daily_volume_usd: f64,
    pub daily_volume_untracked: f64,
    pub total_volume_eth: f64,
    pub total_liquidity_eth: f64,
    pub total_volume_usd: f64,
    pub total_liquidity_usd: f64,
    pub tx_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairHourData {
    pub id: String,
    pub hour_start_unix: i32,
    pub pair: String,
    pub reserve0: f64,
    pub reserve1: f64,
    pub total_supply: Option<f64>,
    pub reserve_usd: f64,
    pub hourly_volume_token0: f64,
    pub hourly_volume_token1: f64,
    pub hourly_volume_usd: f64,
    pub hourly_txns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairDayData {
    pub id: String,
    pub date: i32,
    pub pair_address: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: f64,
    pub reserve1: f64,
    pub total_supply: Option<f64>,
    pub reserve_usd: f64,
    pub daily_volume_token0: f64,
    pub daily_volume_token1: f64,
    pub daily_volume_usd: f64,
    pub daily_txns: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTokenDayData {
    pub id: String,
    pub date: i32,
    pub token: String,
    pub daily_volume_token: f64,
    pub daily_volume_eth: f64,
    pub daily_volume_usd: f64,
    pub daily_txns: u64,
    pub total_liquidity_token: f64,
    pub total_liquidity_eth: f64,
    pub total_liquidity_usd: f64,
    pub price_usd: f64,
}
