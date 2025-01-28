use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseUser {
    pub address: String,
    pub usd_swapped: f64,
}
