use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseSwap {
    pub id: String,
    pub transaction: String,
    pub timestamp: u64,
    pub pair: String,
    pub sender: String,
    pub from: String,
    pub amount0_in: f64,
    pub amount1_in: f64,
    pub amount0_out: f64,
    pub amount1_out: f64,
    pub to: String,
    pub log_index: Option<u64>,
    pub amount_usd: f64,
}
