use alloy::primitives::{Log, LogData};
use fastnum::{udec256, U256, UD256};
use serde::{Deserialize, Serialize};

use crate::handlers::swap::Swap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSwap {
    pub id: String,
    pub transaction: String,
    pub timestamp: U256,
    pub pair: String,
    pub sender: String,
    pub from: String,
    pub amount0_in: UD256,
    pub amount1_in: UD256,
    pub amount0_out: UD256,
    pub amount1_out: UD256,
    pub to: String,
    pub log_index: U256,
    pub amount_usd: UD256,
}

impl DatabaseSwap {
    pub fn from_log(
        log: &alloy::rpc::types::Log<LogData>,
        event: Log<Swap>,
    ) -> Self {
        let transaction = log.transaction_hash.unwrap().to_string();
        Self {
            id: format!(
                "{}-{}",
                transaction,
                log.transaction_index.unwrap()
            ),
            transaction,
            timestamp: U256::from(log.block_timestamp.unwrap()),
            pair: event.address.to_string(),
            sender: event.sender.to_string(),
            from: "".to_owned(),
            amount0_in: udec256!(0),
            amount1_in: udec256!(0),
            amount0_out: udec256!(0),
            amount1_out: udec256!(0),
            to: event.to.to_string(),
            log_index: U256::from(log.log_index.unwrap()),
            amount_usd: udec256!(0),
        }
    }
}
