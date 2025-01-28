use alloy::sol;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePair {
    pub address: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: f64,
    pub reserve1: f64,
    pub total_supply: f64,
    pub reserve_eth: f64,
    pub reserve_usd: f64,
    pub tracked_reserve_eth: f64,
    pub token0_price: f64,
    pub token1_price: f64,
    pub volume_token0: f64,
    pub volume_token1: f64,
    pub volume_usd: f64,
    pub untracked_volume_usd: f64,
    pub tx_count: u64,
    pub created_at_timestamp: u64,
    pub created_at_block_number: u64,
    pub liquidity_provider_count: u64,

    // TODO: find a way to make the relationship
    pub pair_hour_data: Vec<String>,
    pub mints: Vec<String>,
    pub burns: Vec<String>,
    pub swaps: Vec<String>,
}

impl DatabasePair {
    pub fn new(address: String, token0: String, token1: String) -> Self {
        Self {
            address,
            token0,
            token1,
            reserve0: 0.0,
            reserve1: 0.0,
            total_supply: 0.0,
            reserve_eth: 0.0,
            reserve_usd: 0.0,
            tracked_reserve_eth: 0.0,
            token0_price: 0.0,
            token1_price: 0.0,
            volume_token0: 0.0,
            volume_token1: 0.0,
            volume_usd: 0.0,
            untracked_volume_usd: 0.0,
            tx_count: 0,
            created_at_timestamp: 0,
            created_at_block_number: 0,
            liquidity_provider_count: 0,
            pair_hour_data: vec![],
            mints: vec![],
            burns: vec![],
            swaps: vec![],
        }
    }
}
