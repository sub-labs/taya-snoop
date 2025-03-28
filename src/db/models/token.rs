use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::{db::schema::tokens, utils::format::zero_bd};

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseToken {
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

impl DatabaseToken {
    pub fn new(
        address: String,
        symbol: String,
        name: String,
        decimals: i32,
        total_supply: BigDecimal,
    ) -> Self {
        Self {
            id: address.to_lowercase(),
            symbol,
            name,
            decimals,
            total_supply,
            trade_volume: zero_bd(),
            trade_volume_usd: zero_bd(),
            untracked_volume_usd: zero_bd(),
            tx_count: 0,
            total_liquidity: zero_bd(),
            derived_eth: zero_bd(),
        }
    }
}
