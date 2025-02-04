use fastnum::{udec256, UD256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseToken {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u64,
    pub total_supply: UD256,
    pub trade_volume: UD256,
    pub trade_volume_usd: UD256,
    pub untracked_volume_usd: UD256,
    pub tx_count: i64,
    pub total_liquidity: UD256,
    pub derived_eth: UD256,
}

impl DatabaseToken {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: u64,
        total_supply: UD256,
    ) -> Self {
        Self {
            id: address.to_lowercase(),
            symbol,
            name,
            decimals,
            total_supply,
            trade_volume: udec256!(0),
            trade_volume_usd: udec256!(0),
            untracked_volume_usd: udec256!(0),
            tx_count: 0,
            total_liquidity: udec256!(0),
            derived_eth: udec256!(0),
        }
    }
}
