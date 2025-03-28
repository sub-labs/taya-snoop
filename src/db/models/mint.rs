use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::db::schema::mints;

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = mints)]
pub struct DatabaseMint {
    pub id: String,
    pub transaction: String,
    pub timestamp: i32,
    pub pair: String,
    pub to: String,
    pub liquidity: BigDecimal,
    pub sender: String,
    pub amount0: BigDecimal,
    pub amount1: BigDecimal,
    pub log_index: i32,
    pub amount_usd: BigDecimal,
    pub fee_to: String,
    pub fee_liquidity: BigDecimal,
}

/*
impl DatabaseMint {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i64,
        pair: String,
        to: String,
        log_index: i64,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair,
            to,
            liquidity: udec256!(0),
            sender: Address::ZERO.to_string(),
            amount0: udec256!(0),
            amount1: udec256!(0),
            log_index,
            amount_usd: udec256!(0),
            fee_to: Address::ZERO.to_string(),
            fee_liquidity: udec256!(0),
        }
    }
}
 */
