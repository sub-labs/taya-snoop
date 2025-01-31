use alloy::primitives::Address;
use fastnum::{udec256, UD256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMint {
    pub id: String,
    pub transaction: String,
    pub timestamp: i64,
    pub pair: String,
    pub to: String,
    pub liquidity: UD256,
    pub sender: String,
    pub amount0: UD256,
    pub amount1: UD256,
    pub log_index: i64,
    pub amount_usd: UD256,
    pub fee_to: String,
    pub fee_liquidity: UD256,
}

impl DatabaseMint {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i64,
        pair: String,
        to: String,
        log_index: i64,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair,
            to,
            liquidity: udec256!(0),
            sender: Address::ZERO.to_string(),
            amount0: udec256!(0),
            amount1: udec256!(0),
            log_index,
            amount_usd: udec256!(0),
            fee_to: Address::ZERO.to_string(),
            fee_liquidity: udec256!(0),
        }
    }
}
