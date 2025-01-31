use alloy::primitives::{Log, LogData};
use fastnum::{udec256, U256, UD256};
use serde::{Deserialize, Serialize};

use crate::handlers::mint::Mint;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMint {
    pub id: String,
    pub transaction: String,
    pub timestamp: U256,
    pub pair: String,
    pub to: String,
    pub liquidity: UD256,
    pub sender: String,
    pub amount0: UD256,
    pub amount1: UD256,
    pub log_index: U256,
    pub amount_usd: UD256,
    pub fee_to: String,
    pub fee_liquidity: UD256,
}

impl DatabaseMint {
    pub fn from_log(
        log: &alloy::rpc::types::Log<LogData>,
        event: Log<Mint>,
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
            // TODO: fix 'to' and 'liquidity'
            to: "".to_owned(),
            liquidity: udec256!(0),
            sender: event.sender.to_string(),
            // TODO: fix amounts
            amount0: udec256!(0),
            amount1: udec256!(0),
            log_index: U256::from(log.log_index.unwrap()),
            // TODO: fix amount usd and fees
            amount_usd: udec256!(0),
            fee_to: "".to_owned(),
            fee_liquidity: udec256!(0),
        }
    }
}
