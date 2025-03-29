use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::{
    db::{schema::factories, DatabaseKeys},
    utils::format::zero_bd,
};

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = factories)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DatabaseFactory {
    pub id: String,
    pub pair_count: i32,
    pub pairs: Vec<Option<String>>,
    pub total_volume_usd: BigDecimal,
    pub total_volume_eth: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub total_liquidity_usd: BigDecimal,
    pub total_liquidity_eth: BigDecimal,
    pub tx_count: i32,
}

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
            pairs: vec![],
            total_volume_usd: zero_bd(),
            total_volume_eth: zero_bd(),
            untracked_volume_usd: zero_bd(),
            total_liquidity_usd: zero_bd(),
            total_liquidity_eth: zero_bd(),
            tx_count: 0,
        }
    }
}
