use alloy::primitives::Log;
use fastnum::{udec256, UD256};
use serde::{Deserialize, Serialize};

use crate::handlers::pairs::PairCreated;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePair {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: UD256,
    pub reserve1: UD256,
    pub total_supply: UD256,
    pub reserve_eth: UD256,
    pub reserve_usd: UD256,
    pub tracked_reserve_eth: UD256,
    pub token0_price: UD256,
    pub token1_price: UD256,
    pub volume_token0: UD256,
    pub volume_token1: UD256,
    pub volume_usd: UD256,
    pub untracked_volume_usd: UD256,
    pub tx_count: i64,
    pub created_at_timestamp: i64,
    pub created_at_block_number: i64,
    pub liquidity_provider_count: i64,
    // TODO: find a way to make the relationship
    // pub pair_hour_data: Vec<String>,
    // pub mints: Vec<String>,
    // pub burns: Vec<String>,
    // pub swaps: Vec<String>,
}

impl DatabasePair {
    pub fn new(
        event: Log<PairCreated>,
        created_at_timestamp: i64,
        created_at_block_number: i64,
    ) -> Self {
        Self {
            id: event.pair.to_string().to_lowercase(),
            token0: event.token0.to_string().to_lowercase(),
            token1: event.token1.to_string().to_lowercase(),
            reserve0: udec256!(0),
            reserve1: udec256!(0),
            total_supply: udec256!(0),
            reserve_eth: udec256!(0),
            reserve_usd: udec256!(0),
            tracked_reserve_eth: udec256!(0),
            token0_price: udec256!(0),
            token1_price: udec256!(0),
            volume_token0: udec256!(0),
            volume_token1: udec256!(0),
            volume_usd: udec256!(0),
            untracked_volume_usd: udec256!(0),
            tx_count: 0,
            created_at_timestamp,
            created_at_block_number,
            liquidity_provider_count: 0,
        }
    }
}
