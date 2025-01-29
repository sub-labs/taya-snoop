use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSyncState {
    pub id: String,
    pub last_block_indexed: i64,
}
