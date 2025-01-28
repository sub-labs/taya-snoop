use alloy::rpc::types::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseLog {
    pub address: String,
    pub block_number: u64,
    pub block_hash: String,
    pub chain: u64,
    pub data: String,
    pub log_index: u64,
    pub removed: bool,
    pub timestamp: u64,
    pub topic0: String,
    pub topic1: Option<String>,
    pub topic2: Option<String>,
    pub topic3: Option<String>,
    pub transaction_hash: String,
    pub transaction_log_index: Option<u64>,
}

impl DatabaseLog {
    pub fn from_rpc(log: &Log, chain: u64) -> Self {
        let topic0 = if log.topic0().is_none() {
            String::from("0x")
        } else {
            log.topic0().unwrap().to_string()
        };

        let topics = log.topics();

        let topic1 = if topics.len() < 2 {
            None
        } else {
            Some(topics[1].to_string())
        };

        let topic2 = if topics.len() < 3 {
            None
        } else {
            Some(topics[2].to_string())
        };

        let topic3 = if topics.len() < 4 {
            None
        } else {
            Some(topics[3].to_string())
        };

        let transaction_log_index = log
            .transaction_index
            .map(|transaction_index| transaction_index);

        let timestamp = match log.block_timestamp {
            Some(timestamp) => timestamp,
            None => 0,
        };

        Self {
            address: log.address().to_string(),
            block_hash: log.block_hash.unwrap().to_string(),
            block_number: log.block_number.unwrap(),
            chain,
            data: log.data().data.to_string(),
            log_index: log.log_index.unwrap(),
            removed: log.removed,
            timestamp,
            topic0,
            topic1,
            topic2,
            topic3,
            transaction_hash: log.transaction_hash.unwrap().to_string(),
            transaction_log_index,
        }
    }
}
