use bigdecimal::{BigDecimal, Zero};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i64,
    pub total_supply: String,
    pub trade_volume: BigDecimal,
    pub trade_volume_usd: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub tx_count: i64,
    pub total_liquidity: BigDecimal,
    pub derived_eth: BigDecimal,
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
            id: address.to_lowercase(),
            symbol,
            name,
            decimals,
            total_supply,
            trade_volume: BigDecimal::zero(),
            trade_volume_usd: BigDecimal::zero(),
            untracked_volume_usd: BigDecimal::zero(),
            tx_count: 0,
            total_liquidity: BigDecimal::zero(),
            derived_eth: BigDecimal::zero(),
        }
    }
}
