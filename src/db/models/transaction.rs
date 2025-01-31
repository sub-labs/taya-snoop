use fastnum::U256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseTransaction {
    pub hash: String,
    pub block_number: U256,
    pub timestamp: U256,
    pub mints: Vec<String>,
    pub burns: Vec<String>,
    pub swaps: Vec<String>,
}
