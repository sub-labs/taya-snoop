use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::db::schema::pairs;

#[derive(Queryable, Insertable, Debug, Clone)]
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

/*
impl DatabasePair {
    pub fn new(
        event: Log<PairCreated>,
        created_at_timestamp: i64,
        created_at_block_number: i64,
    ) -> Self {
        Self {
            id: event.pair.to_string().to_lowercase(),
            pair: event.pair.to_string().to_lowercase(),
            token0: event.token0.to_string().to_lowercase(),
            token1: event.token1.to_string().to_lowercase(),
            reserve0: udec256!(0),
            reserve1: udec256!(0),
            total_supply: udec256!(0),
            reserve_eth: udec256!(0),
            reserve_usd: udec256!(0),
            tracked_reserve_eth: udec256!(0),
            token0_price: udec256!(0),
            token1_price: udec256!(0),
            volume_token0: udec256!(0),
            volume_token1: udec256!(0),
            volume_usd: udec256!(0),
            untracked_volume_usd: udec256!(0),
            tx_count: 0,
            created_at_timestamp,
            created_at_block_number,
            liquidity_provider_count: 0,
        }
    }
}
 */
