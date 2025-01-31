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
