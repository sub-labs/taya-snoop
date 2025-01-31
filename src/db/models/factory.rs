use fastnum::{u256, udec256, U256, UD256};
use serde::{Deserialize, Serialize};

use crate::db::DatabaseKeys;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseFactory {
    pub id: String,
    pub pair_count: i32,
    pub total_volume_usd: UD256,
    pub total_volume_eth: UD256,
    pub untracked_volume_usd: UD256,
    pub total_liquidity_usd: UD256,
    pub total_liquidity_eth: UD256,
    pub tx_count: U256,
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
            total_volume_usd: udec256!(0),
            total_volume_eth: udec256!(0),
            untracked_volume_usd: udec256!(0),
            total_liquidity_usd: udec256!(0),
            total_liquidity_eth: udec256!(0),
            tx_count: u256!(0),
            pairs: vec![],
        }
    }
}
