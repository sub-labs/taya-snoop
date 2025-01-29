use alloy::primitives::{Log, LogData};
use serde::{Deserialize, Serialize};

use super::pair::Burn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBurn {
    pub id: String,
    pub transaction: String,
    pub timestamp: u64,
    pub pair: String,
    pub liquidity: f64,
    pub sender: Option<String>,
    pub amount0: Option<f64>,
    pub amount1: Option<f64>,
    pub to: Option<String>,
    pub log_index: Option<u64>,
    pub amount_usd: Option<f64>,
    pub needs_complete: bool,
    pub fee_to: Option<String>,
    pub fee_liquidity: Option<f64>,
}

impl DatabaseBurn {
    pub fn from_log(
        log: &alloy::rpc::types::Log<LogData>,
        event: Log<Burn>,
    ) -> Self {
        let transaction = log.transaction_hash.unwrap().to_string();

        Self {
            id: format!(
                "{}-{}",
                transaction,
                log.transaction_index.unwrap()
            ),
            transaction,
            timestamp: log.block_timestamp.unwrap(),
            pair: event.address.to_string(),
            // TODO: fix 'to' and 'liquidity'
            to: Some("".to_owned()),
            liquidity: 0.0,
            sender: Some(event.sender.to_string()),
            // TODO: fix amounts
            amount0: Some(0.0),
            amount1: Some(0.0),
            log_index: log.log_index,
            // TODO: fix amount usd and fees
            amount_usd: Some(0.0),
            fee_to: Some("".to_owned()),
            fee_liquidity: Some(0.0),
            // TODO: fix 'needs_complete'
            needs_complete: false,
        }
    }
}
