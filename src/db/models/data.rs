use fastnum::{udec256, UD256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFactoryDayData {
    pub id: String,
    pub date: i64,
    pub daily_volume_eth: UD256,
    pub daily_volume_usd: UD256,
    pub daily_volume_untracked: UD256,
    pub total_volume_eth: UD256,
    pub total_liquidity_eth: UD256,
    pub total_volume_usd: UD256,
    pub total_liquidity_usd: UD256,
    pub tx_count: i64,
}

impl DatabaseFactoryDayData {
    pub fn new(day_id: String, timestamp: i64) -> Self {
        Self {
            id: day_id,
            date: timestamp,
            daily_volume_eth: udec256!(0),
            daily_volume_usd: udec256!(0),
            daily_volume_untracked: udec256!(0),
            total_volume_eth: udec256!(0),
            total_liquidity_eth: udec256!(0),
            total_volume_usd: udec256!(0),
            total_liquidity_usd: udec256!(0),
            tx_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairHourData {
    pub id: String,
    pub hour_start_unix: i64,
    pub pair: String,
    pub reserve0: UD256,
    pub reserve1: UD256,
    pub total_supply: UD256,
    pub reserve_usd: UD256,
    pub hourly_volume_token0: UD256,
    pub hourly_volume_token1: UD256,
    pub hourly_volume_usd: UD256,
    pub hourly_txns: i64,
}

impl DatabasePairHourData {
    pub fn new(
        hour_pair_id: String,
        hour_start_unix: i64,
        pair_address: String,
    ) -> Self {
        Self {
            id: hour_pair_id,
            hour_start_unix,
            pair: pair_address,
            reserve0: udec256!(0),
            reserve1: udec256!(0),
            total_supply: udec256!(0),
            reserve_usd: udec256!(0),
            hourly_volume_token0: udec256!(0),
            hourly_volume_token1: udec256!(0),
            hourly_volume_usd: udec256!(0),
            hourly_txns: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePairDayData {
    pub id: String,
    pub date: i64,
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
    pub daily_txns: i64,
}

impl DatabasePairDayData {
    pub fn new(
        day_pair_id: String,
        timestamp: i64,
        pair_address: String,
        token0: String,
        token1: String,
    ) -> Self {
        Self {
            id: day_pair_id,
            date: timestamp,
            pair_address,
            token0,
            token1,
            reserve0: udec256!(0),
            reserve1: udec256!(0),
            total_supply: udec256!(0),
            reserve_usd: udec256!(0),
            daily_volume_token0: udec256!(0),
            daily_volume_token1: udec256!(0),
            daily_volume_usd: udec256!(0),
            daily_txns: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTokenDayData {
    pub id: String,
    pub date: i64,
    pub token: String,
    pub daily_volume_token: UD256,
    pub daily_volume_eth: UD256,
    pub daily_volume_usd: UD256,
    pub daily_txns: i64,
    pub total_liquidity_token: UD256,
    pub total_liquidity_eth: UD256,
    pub total_liquidity_usd: UD256,
    pub price_usd: UD256,
}

impl DatabaseTokenDayData {
    pub fn new(
        token_day_id: String,
        day_start_time: i64,
        token: String,
        price_usd: UD256,
    ) -> Self {
        Self {
            id: token_day_id,
            date: day_start_time,
            token,
            daily_volume_token: udec256!(0),
            daily_volume_eth: udec256!(0),
            daily_volume_usd: udec256!(0),
            daily_txns: 0,
            total_liquidity_token: udec256!(0),
            total_liquidity_eth: udec256!(0),
            total_liquidity_usd: udec256!(0),
            price_usd,
        }
    }
}
