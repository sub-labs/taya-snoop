use fastnum::{U256, UD256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseDexDayData {
    pub id: String,
    pub date: i32,
    pub daily_volume_eth: UD256,
    pub daily_volume_usd: UD256,
    pub daily_volume_untracked: UD256,
    pub total_volume_eth: UD256,
    pub total_liquidity_eth: UD256,
    pub total_volume_usd: UD256,
    pub total_liquidity_usd: UD256,
    pub tx_count: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairHourData {
    pub id: String,
    pub hour_start_unix: i32,
    pub pair: String,
    pub reserve0: UD256,
    pub reserve1: UD256,
    pub total_supply: Option<UD256>,
    pub reserve_usd: UD256,
    pub hourly_volume_token0: UD256,
    pub hourly_volume_token1: UD256,
    pub hourly_volume_usd: UD256,
    pub hourly_txns: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairDayData {
    pub id: String,
    pub date: i32,
    pub pair_address: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: UD256,
    pub reserve1: UD256,
    pub total_supply: UD256,
    pub reserve_usd: UD256,
    pub daily_volume_token0: UD256,
    pub daily_volume_token1: UD256,
    pub daily_volume_usd: UD256,
    pub daily_txns: UD256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTokenDayData {
    pub id: String,
    pub date: i32,
    pub token: String,
    pub daily_volume_token: UD256,
    pub daily_volume_eth: UD256,
    pub daily_volume_usd: UD256,
    pub daily_txns: U256,
    pub total_liquidity_token: UD256,
    pub total_liquidity_eth: UD256,
    pub total_liquidity_usd: UD256,
    pub price_usd: UD256,
}
