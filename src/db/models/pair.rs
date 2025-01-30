use alloy::{primitives::Log, sol};
use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};

use super::factory::PairCreated;

sol! {
    event Mint(address indexed sender, uint amount0, uint amount1);
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
    event Sync(uint112 reserve0, uint112 reserve1);
    event Transfer(address indexed,address indexed,uint256);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePair {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: BigDecimal,
    pub reserve1: BigDecimal,
    pub total_supply: BigDecimal,
    pub reserve_eth: BigDecimal,
    pub reserve_usd: BigDecimal,
    pub tracked_reserve_eth: BigDecimal,
    pub token0_price: BigDecimal,
    pub token1_price: BigDecimal,
    pub volume_token0: BigDecimal,
    pub volume_token1: BigDecimal,
    pub volume_usd: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
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
            token0: event.token0.to_string(),
            token1: event.token1.to_string(),
            reserve0: BigDecimal::zero(),
            reserve1: BigDecimal::zero(),
            total_supply: BigDecimal::zero(),
            reserve_eth: BigDecimal::zero(),
            reserve_usd: BigDecimal::zero(),
            tracked_reserve_eth: BigDecimal::zero(),
            token0_price: BigDecimal::zero(),
            token1_price: BigDecimal::zero(),
            volume_token0: BigDecimal::zero(),
            volume_token1: BigDecimal::zero(),
            volume_usd: BigDecimal::zero(),
            untracked_volume_usd: BigDecimal::zero(),
            tx_count: 0,
            created_at_timestamp,
            created_at_block_number,
            liquidity_provider_count: 0,
        }
    }
}
