use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    pub total_supply: String,
    pub trade_volume: f64,
    pub trade_volume_usd: f64,
    pub untracked_volume_usd: f64,
    pub tx_count: i64,
    pub total_liquidity: f64,
    pub derived_eth: f64,
}

impl DatabaseToken {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: i64,
        total_supply: String,
    ) -> Self {
        Self {
            id: address,
            symbol,
            name,
            decimals,
            total_supply,
            trade_volume: 0.0,
            trade_volume_usd: 0.0,
            untracked_volume_usd: 0.0,
            tx_count: 0,
            total_liquidity: 0.0,
            derived_eth: 0.0,
        }
    }
}
