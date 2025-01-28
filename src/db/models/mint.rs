use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMint {
    pub id: String,
    pub timestamp: u64,
    pub pair: String,
    pub to: String,
    pub liquidity: f64,
    pub sender: Option<String>,
    pub amount0: Option<f64>,
    pub amount1: Option<f64>,
    pub log_index: Option<u64>,
    pub amount_usd: Option<f64>,
    pub fee_to: Option<String>,
    pub fee_liquidity: Option<f64>,
}
