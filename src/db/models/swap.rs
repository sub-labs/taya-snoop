use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::db::schema::swaps;

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = swaps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DatabaseSwap {
    pub id: String,
    pub transaction: String,
    pub timestamp: i32,
    pub pair: String,
    pub sender: String,
    pub from: String,
    pub amount0_in: BigDecimal,
    pub amount1_in: BigDecimal,
    pub amount0_out: BigDecimal,
    pub amount1_out: BigDecimal,
    pub to: String,
    pub log_index: i32,
    pub amount_usd: BigDecimal,
}

impl DatabaseSwap {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i32,
        pair: String,
        sender: String,
        from: String,
        amount0_in: BigDecimal,
        amount1_in: BigDecimal,
        amount0_out: BigDecimal,
        amount1_out: BigDecimal,
        log_index: i32,
        amount_usd: BigDecimal,
        to: String,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair,
            sender,
            from,
            amount0_in,
            amount1_in,
            amount0_out,
            amount1_out,
            to,
            log_index,
            amount_usd,
        }
    }
}
