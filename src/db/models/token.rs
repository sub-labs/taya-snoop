use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::db::schema::tokens;

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = tokens)]
pub struct Token {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub total_supply: BigDecimal,
    pub trade_volume: BigDecimal,
    pub trade_volume_usd: BigDecimal,
    pub untracked_volume_usd: BigDecimal,
    pub tx_count: i32,
    pub total_liquidity: BigDecimal,
    pub derived_eth: BigDecimal,
}

/*
impl DatabaseToken {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: u64,
        total_supply: UD256,
    ) -> Self {
        Self {
            id: address.to_lowercase(),
            symbol,
            name,
            decimals,
            total_supply,
            trade_volume: udec256!(0),
            trade_volume_usd: udec256!(0),
            untracked_volume_usd: udec256!(0),
            tx_count: 0,
            total_liquidity: udec256!(0),
            derived_eth: udec256!(0),
        }
    }
}
 */
