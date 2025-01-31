use alloy::primitives::Address;
use fastnum::{udec256, UD256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

pub struct BurnData {
    pub sender: Option<String>,
    pub liquidity: UD256,
    pub pair: String,
    pub to: Option<String>,
    pub needs_complete: bool,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBurn {
    pub id: String,
    pub transaction: String,
    pub timestamp: i64,
    pub pair: String,
    pub liquidity: UD256,
    pub sender: Option<String>,
    pub amount0: UD256,
    pub amount1: UD256,
    pub to: Option<String>,
    pub log_index: i64,
    pub amount_usd: UD256,
    pub needs_complete: bool,
    pub fee_to: String,
    pub fee_liquidity: UD256,
}

impl DatabaseBurn {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i64,
        log_index: i64,
        data: BurnData,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair: data.pair,
            to: data.to,
            liquidity: data.liquidity,
            sender: data.sender,
            amount0: udec256!(0),
            amount1: udec256!(0),
            log_index,
            amount_usd: udec256!(0),
            fee_to: Address::ZERO.to_string(),
            fee_liquidity: udec256!(0),
            needs_complete: data.needs_complete,
        }
    }
}
