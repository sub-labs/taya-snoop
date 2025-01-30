use alloy::sol;
use serde::{Deserialize, Serialize};

use crate::db::DatabaseKeys;

sol! {
    event PairCreated(address indexed token0, address indexed token1, address pair, uint);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFactory {
    pub id: String,
    pub pair_count: i32,
    pub total_volume_usd: f64,
    pub total_volume_eth: f64,
    pub untracked_volume_usd: f64,
    pub total_liquidity_usd: f64,
    pub total_liquidity_eth: f64,
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
            total_volume_usd: 0.0,
            total_volume_eth: 0.0,
            untracked_volume_usd: 0.0,
            total_liquidity_usd: 0.0,
            total_liquidity_eth: 0.0,
            tx_count: 0,
            pairs: vec![],
        }
    }
}
