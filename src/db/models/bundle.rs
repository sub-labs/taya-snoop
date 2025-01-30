use serde::{Deserialize, Serialize};

use crate::db::DatabaseKeys;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBundle {
    pub id: String,
    pub eth_price: f64,
}

impl Default for DatabaseBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseBundle {
    pub fn new() -> Self {
        Self {
            id: DatabaseKeys::Bundle.as_str().to_owned(),
            eth_price: 0.0,
        }
    }
}
