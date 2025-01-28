use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBundle {
    pub id: String,
    pub eth_price: f64,
}
