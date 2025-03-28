use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::db::schema::factories;

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = factories)]
pub struct DatabaseDatabaseFactory {
    pub id: String,
    pub pair_count: i32,
    pub total_volume_usd: BigDecimal,
    pub total_volume_eth: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub total_liquidity_usd: BigDecimal,
    pub total_liquidity_eth: BigDecimal,
    pub tx_count: i32,
}
/*
impl Default for DatabaseFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseFactory {
    pub fn new() -> Self {
        Self {
            id: DatabaseKeys::Factory.as_str().to_owned(),
            pair_count: 0,
            total_volume_usd: udec256!(0),
            total_volume_eth: udec256!(0),
            untracked_volume_usd: udec256!(0),
            total_liquidity_usd: udec256!(0),
            total_liquidity_eth: udec256!(0),
            tx_count: 0,
        }
    }
}
 */
