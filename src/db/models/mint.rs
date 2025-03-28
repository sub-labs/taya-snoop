use alloy::primitives::Address;
use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};
use field_count::FieldCount;

use crate::{db::schema::mints, utils::format::zero_bd};

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
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

impl DatabaseMint {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i32,
        pair: String,
        to: String,
        log_index: i32,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair,
            to,
            liquidity: zero_bd(),
            sender: Address::ZERO.to_string(),
            amount0: zero_bd(),
            amount1: zero_bd(),
            log_index,
            amount_usd: zero_bd(),
            fee_to: Address::ZERO.to_string(),
            fee_liquidity: zero_bd(),
        }
    }
}
