use alloy::sol;
use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};

use crate::db::DatabaseKeys;

sol! {
    event PairCreated(address indexed token0, address indexed token1, address pair, uint);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFactory {
    pub id: String,
    pub pair_count: i32,
    pub total_volume_usd: BigDecimal,
    pub total_volume_eth: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub total_liquidity_usd: BigDecimal,
    pub total_liquidity_eth: BigDecimal,
    pub tx_count: i64,
    pub pairs: Vec<String>,
}

impl Default for DatabaseFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseFactory {
    pub fn new() -> Self {
        Self {
            id: DatabaseKeys::Factory.as_str().to_owned(),
            pair_count: 0,
            total_volume_usd: BigDecimal::zero(),
            total_volume_eth: BigDecimal::zero(),
            untracked_volume_usd: BigDecimal::zero(),
            total_liquidity_usd: BigDecimal::zero(),
            total_liquidity_eth: BigDecimal::zero(),
            tx_count: 0,
            pairs: vec![],
        }
    }
}
