use fastnum::UD256;
use serde::{Deserialize, Serialize};

pub struct SwapAmounts {
    pub amount0_in: UD256,
    pub amount1_in: UD256,
    pub amount0_out: UD256,
    pub amount1_out: UD256,
    pub amount_usd: UD256,
}

pub struct SwapData {
    pub pair: String,
    pub sender: String,
    pub to: String,
    pub from: String,
    pub log_index: i64,
    pub transaction: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSwap {
    pub id: String,
    pub transaction: String,
    pub timestamp: i64,
    pub pair: String,
    pub sender: String,
    pub from: String,
    pub amount0_in: UD256,
    pub amount1_in: UD256,
    pub amount0_out: UD256,
    pub amount1_out: UD256,
    pub to: String,
    pub log_index: i64,
    pub amount_usd: UD256,
}

impl DatabaseSwap {
    pub fn new(id: String, data: SwapData, amounts: SwapAmounts) -> Self {
        Self {
            id,
            transaction: data.transaction,
            timestamp: data.timestamp,
            pair: data.pair,
            sender: data.sender,
            from: data.from,
            amount0_in: amounts.amount0_in,
            amount1_in: amounts.amount1_in,
            amount0_out: amounts.amount0_out,
            amount1_out: amounts.amount1_out,
            to: data.to,
            log_index: data.log_index,
            amount_usd: amounts.amount_usd,
        }
    }
}
