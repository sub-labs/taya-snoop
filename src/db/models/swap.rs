use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::db::schema::swaps;

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = swaps)]
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

/*
impl DatabaseSwap {
    pub fn new(id: String, data: SwapData, amounts: SwapAmounts) -> Self {
        Self {
            id,
            transaction: data.transaction,
            timestamp: data.timestamp,
            pair: data.pair,
            sender: data.sender,
            from: data.from,
            amount0_in: amounts.amount0_in,
            amount1_in: amounts.amount1_in,
            amount0_out: amounts.amount0_out,
            amount1_out: amounts.amount1_out,
            to: data.to,
            log_index: data.log_index,
            amount_usd: amounts.amount_usd,
        }
    }
}
 */
