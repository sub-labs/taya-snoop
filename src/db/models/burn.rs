use alloy::primitives::Address;
use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::{db::schema::burns, utils::format::zero_bd};

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = burns)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseBurn {
    pub id: String,
    pub transaction: String,
    pub timestamp: i32,
    pub pair: String,
    pub liquidity: BigDecimal,
    pub sender: String,
    pub amount0: BigDecimal,
    pub amount1: BigDecimal,
    pub to: String,
    pub log_index: i32,
    pub amount_usd: BigDecimal,
    pub needs_complete: bool,
    pub fee_to: String,
    pub fee_liquidity: BigDecimal,
}

impl DatabaseBurn {
    pub fn new(
        id: String,
        transaction: String,
        timestamp: i32,
        log_index: i32,
        pair: String,
        to: String,
        liquidity: BigDecimal,
        sender: String,
        needs_complete: bool,
    ) -> Self {
        Self {
            id,
            transaction,
            timestamp,
            pair,
            to,
            liquidity,
            sender,
            amount0: zero_bd(),
            amount1: zero_bd(),
            log_index,
            amount_usd: zero_bd(),
            fee_to: Address::ZERO.to_string(),
            fee_liquidity: zero_bd(),
            needs_complete,
        }
    }
}
