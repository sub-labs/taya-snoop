use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::{
    db::schema::{
        dex_day_data, pair_day_data, pair_hour_data, token_day_data,
    },
    utils::format::zero_bd,
};

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = dex_day_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseDexDayData {
    pub id: String,
    pub date: i32,
    pub daily_volume_eth: BigDecimal,
    pub daily_volume_usd: BigDecimal,
    pub daily_volume_untracked: BigDecimal,
    pub total_volume_eth: BigDecimal,
    pub total_liquidity_eth: BigDecimal,
    pub total_volume_usd: BigDecimal,
    pub total_liquidity_usd: BigDecimal,
    pub tx_count: i32,
}

impl DatabaseDexDayData {
    pub fn new(day_id: String, timestamp: i32) -> Self {
        Self {
            id: day_id,
            date: timestamp,
            daily_volume_eth: zero_bd(),
            daily_volume_usd: zero_bd(),
            daily_volume_untracked: zero_bd(),
            total_volume_eth: zero_bd(),
            total_liquidity_eth: zero_bd(),
            total_volume_usd: zero_bd(),
            total_liquidity_usd: zero_bd(),
            tx_count: 0,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = pair_hour_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabasePairHourData {
    pub id: String,
    pub hour_start_unix: i32,
    pub pair: String,
    pub reserve0: BigDecimal,
    pub reserve1: BigDecimal,
    pub total_supply: BigDecimal,
    pub reserve_usd: BigDecimal,
    pub hourly_volume_token0: BigDecimal,
    pub hourly_volume_token1: BigDecimal,
    pub hourly_volume_usd: BigDecimal,
    pub hourly_txns: i32,
}

impl DatabasePairHourData {
    pub fn new(
        hour_pair_id: String,
        hour_start_unix: i32,
        pair_address: String,
    ) -> Self {
        Self {
            id: hour_pair_id,
            hour_start_unix,
            pair: pair_address,
            reserve0: zero_bd(),
            reserve1: zero_bd(),
            total_supply: zero_bd(),
            reserve_usd: zero_bd(),
            hourly_volume_token0: zero_bd(),
            hourly_volume_token1: zero_bd(),
            hourly_volume_usd: zero_bd(),
            hourly_txns: 0,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = pair_day_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabasePairDayData {
    pub id: String,
    pub date: i32,
    pub pair_address: String,
    pub token0: String,
    pub token1: String,
    pub reserve0: BigDecimal,
    pub reserve1: BigDecimal,
    pub total_supply: BigDecimal,
    pub reserve_usd: BigDecimal,
    pub daily_volume_token0: BigDecimal,
    pub daily_volume_token1: BigDecimal,
    pub daily_volume_usd: BigDecimal,
    pub daily_txns: i32,
}

impl DatabasePairDayData {
    pub fn new(
        day_pair_id: String,
        timestamp: i32,
        pair_address: String,
        token0: String,
        token1: String,
    ) -> Self {
        Self {
            id: day_pair_id,
            date: timestamp,
            pair_address,
            token0,
            token1,
            reserve0: zero_bd(),
            reserve1: zero_bd(),
            total_supply: zero_bd(),
            reserve_usd: zero_bd(),
            daily_volume_token0: zero_bd(),
            daily_volume_token1: zero_bd(),
            daily_volume_usd: zero_bd(),
            daily_txns: 0,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = token_day_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseTokenDayData {
    pub id: String,
    pub date: i32,
    pub token: String,
    pub daily_volume_token: BigDecimal,
    pub daily_volume_eth: BigDecimal,
    pub daily_volume_usd: BigDecimal,
    pub daily_txns: i32,
    pub total_liquidity_token: BigDecimal,
    pub total_liquidity_eth: BigDecimal,
    pub total_liquidity_usd: BigDecimal,
    pub price_usd: BigDecimal,
}

impl DatabaseTokenDayData {
    pub fn new(
        token_day_id: String,
        day_start_time: i32,
        token: String,
        price_usd: BigDecimal,
    ) -> Self {
        Self {
            id: token_day_id,
            date: day_start_time,
            token,
            daily_volume_token: zero_bd(),
            daily_volume_eth: zero_bd(),
            daily_volume_usd: zero_bd(),
            daily_txns: 0,
            total_liquidity_token: zero_bd(),
            total_liquidity_eth: zero_bd(),
            total_liquidity_usd: zero_bd(),
            price_usd,
        }
    }
}
