use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTransaction {
    pub hash: String,
    pub block_number: i64,
    pub timestamp: i64,
    pub mints: Vec<String>,
    pub burns: Vec<String>,
    pub swaps: Vec<String>,
}

impl DatabaseTransaction {
    pub fn new(hash: String, block_number: i64, timestamp: i64) -> Self {
        Self {
            hash,
            block_number,
            timestamp,
            mints: Vec::new(),
            burns: Vec::new(),
            swaps: Vec::new(),
        }
    }
}
