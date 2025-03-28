use alloy::primitives::Log;
use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};
use field_count::FieldCount;

use crate::{
    abi::factory::FACTORY::PairCreated, db::schema::pairs,
    utils::format::zero_bd,
};

#[derive(Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = pairs)]
pub struct DatabasePair {
    pub id: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: BigDecimal,
    pub reserve1: BigDecimal,
    pub total_supply: BigDecimal,
    pub reserve_eth: BigDecimal,
    pub reserve_usd: BigDecimal,
    pub tracked_reserve_eth: BigDecimal,
    pub token0_price: BigDecimal,
    pub token1_price: BigDecimal,
    pub volume_token0: BigDecimal,
    pub volume_token1: BigDecimal,
    pub volume_usd: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub tx_count: i32,
    pub created_at_timestamp: i32,
    pub created_at_block_number: i32,
    pub liquidity_provider_count: i32,
}

impl DatabasePair {
    pub fn new(
        event: Log<PairCreated>,
        created_at_timestamp: i32,
        created_at_block_number: i32,
    ) -> Self {
        Self {
            id: event.pair.to_string().to_lowercase(),
            token0: event.token0.to_string().to_lowercase(),
            token1: event.token1.to_string().to_lowercase(),
            reserve0: zero_bd(),
            reserve1: zero_bd(),
            total_supply: zero_bd(),
            reserve_eth: zero_bd(),
            reserve_usd: zero_bd(),
            tracked_reserve_eth: zero_bd(),
            token0_price: zero_bd(),
            token1_price: zero_bd(),
            volume_token0: zero_bd(),
            volume_token1: zero_bd(),
            volume_usd: zero_bd(),
            untracked_volume_usd: zero_bd(),
            tx_count: 0,
            created_at_timestamp,
            created_at_block_number,
            liquidity_provider_count: 0,
        }
    }
}
