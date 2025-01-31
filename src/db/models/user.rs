use fastnum::UD256;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub id: String,
    pub usd_swapped: UD256,
}
