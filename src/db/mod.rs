pub mod models;

use crate::chains::Chain;

use log::*;

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db_url: String,
}

impl Database {
    pub async fn new(db_url: String, chain: Chain) -> Self {
        info!("Starting database service");

        Self { chain, db_url }
    }

    pub fn get_last_block_indexed(&self) -> i64 {
        0
    }

    pub fn update_last_block_indexed(&self, new_last_block_number: i64) {}
}
