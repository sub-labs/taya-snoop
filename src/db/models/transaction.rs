use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTransaction {
    pub hash: String,
    pub block_number: u64,
    pub timestamp: u64,
    pub mints: Vec<String>,
    pub burns: Vec<String>,
    pub swaps: Vec<String>,
}
