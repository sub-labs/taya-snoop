use crate::{
    configs::Config,
    db::{
        models::{
            data::{
                DatabaseFactoryDayData, DatabasePairDayData,
                DatabasePairHourData, DatabaseTokenDayData,
            },
            pair::DatabasePair,
            token::DatabaseToken,
        },
        Database,
    },
    rpc::Rpc,
};
use alloy::{primitives::Address, rpc::types::Log};
use fastnum::{decimal::Context, u256, udec256, U256, UD256};

pub const MINIMUM_USD_THRESHOLD_NEW_PAIRS: UD256 = udec256!(400000);

pub const MINIMUM_LIQUIDITY_THRESHOLD_ETH: UD256 = udec256!(2);

pub const WHITELIST_TOKENS: [&str; 4] = [
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
    "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
    "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
];

pub const WETH_ADDRESS: &str =
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub const DAI_WETH_PAIR: &str =
    "0xa478c2975ab1ea89e8196811f51a7b7ade33eb11";
pub const USDC_WETH_PAIR: &str =
    "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc";
pub const USDT_WETH_PAIR: &str =
    "0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852";

pub async fn get_eth_price_usd(db: &Database) -> UD256 {
    let dai_pair = db.get_pair(DAI_WETH_PAIR).await;
    let usdc_pair = db.get_pair(USDC_WETH_PAIR).await;
    let usdt_pair = db.get_pair(USDT_WETH_PAIR).await;

    match (dai_pair, usdc_pair, usdt_pair) {
        (Some(dai), Some(usdc), Some(usdt)) => {
            let total_liquidity_eth =
                dai.reserve1.add(usdt.reserve0).add(usdc.reserve1);

            if total_liquidity_eth.eq(&udec256!(0)) {
                return udec256!(0);
            }

            let dai_weight = dai.reserve1.div(total_liquidity_eth);
            let usdc_weight = usdc.reserve1.div(total_liquidity_eth);
            let usdt_weight = usdt.reserve0.div(total_liquidity_eth);

            dai.token0_price
                .mul(dai_weight)
                .add(usdc.token0_price.mul(usdc_weight))
                .add(usdt.token1_price.mul(usdt_weight))
        }
        (Some(dai), Some(usdc), None) => {
            let total_liquidity_eth = dai.reserve1.add(usdc.reserve1);

            if total_liquidity_eth.eq(&udec256!(0)) {
                return udec256!(0);
            }

            let dai_weight = dai.reserve1.div(total_liquidity_eth);
            let usdc_weight = usdc.reserve1.div(total_liquidity_eth);

            dai.token0_price
                .mul(dai_weight)
                .add(usdc.token0_price.mul(usdc_weight))
        }
        (_, Some(usdc), _) => usdc.token0_price,
        // No pairs
        _ => udec256!(0),
    }
}

pub async fn find_eth_per_token(
    token: &DatabaseToken,
    rpc: &Rpc,
    db: &Database,
    config: &Config,
) -> UD256 {
    if token.id == WETH_ADDRESS {
        return udec256!(1);
    }

    // Loop through a set of whitelisted tokens to check if there is any pair for this token.
    for whitelist_token in WHITELIST_TOKENS {
        let pair_address = rpc
            .get_pair_for_tokens(
                token.id.clone(),
                whitelist_token.to_owned().to_lowercase(),
                config,
            )
            .await;

        if pair_address != Address::ZERO.to_string() {
            let pair = db.get_pair(&pair_address).await;

            if pair.is_none() {
                continue;
            }

            let pair = pair.unwrap();

            if pair.token0 == token.id
                && pair.reserve_eth.gt(&MINIMUM_LIQUIDITY_THRESHOLD_ETH)
            {
                let token0 = db.get_token(&pair.token0).await;
                if token0.is_none() {
                    continue;
                }

                let token0 = token0.unwrap();

                return pair.token0_price.mul(token0.derived_eth);
            }

            if pair.token1 == token.id
                && pair.reserve_eth.gt(&MINIMUM_LIQUIDITY_THRESHOLD_ETH)
            {
                let token1 = db.get_token(&pair.token1).await;
                if token1.is_none() {
                    continue;
                }

                let token1 = token1.unwrap();

                return pair.token1_price.mul(token1.derived_eth);
            }
        }
    }

    udec256!(0)
}

pub async fn get_tracked_volume_usd(
    token_amount0: UD256,
    token0: &DatabaseToken,
    token_amount1: UD256,
    token1: &DatabaseToken,
    pair: &DatabasePair,
    db: &Database,
) -> UD256 {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.mul(bundle.eth_price);
    let price1 = token1.derived_eth.mul(bundle.eth_price);

    if pair.liquidity_provider_count < 5 {
        let reserve0_usd = pair.reserve0.mul(price0);
        let reserve1_usd = pair.reserve1.mul(price1);

        if WHITELIST_TOKENS.contains(&token0.id.as_str())
            && WHITELIST_TOKENS.contains(&token1.id.as_str())
            && reserve0_usd
                .add(reserve1_usd)
                .lt(&MINIMUM_USD_THRESHOLD_NEW_PAIRS)
        {
            return udec256!(0);
        }

        if WHITELIST_TOKENS.contains(&token0.id.as_str())
            && !WHITELIST_TOKENS.contains(&token1.id.as_str())
            && reserve0_usd
                .mul(udec256!(2))
                .lt(&MINIMUM_USD_THRESHOLD_NEW_PAIRS)
        {
            return udec256!(0);
        }

        if !WHITELIST_TOKENS.contains(&token0.id.as_str())
            && WHITELIST_TOKENS.contains(&token1.id.as_str())
            && reserve1_usd
                .mul(udec256!(2))
                .lt(&MINIMUM_USD_THRESHOLD_NEW_PAIRS)
        {
            return udec256!(0);
        }
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0
            .mul(price0)
            .add(token_amount1.mul(price1))
            .div(udec256!(2));
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0.mul(price0);
    }

    if !WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount1.mul(price1);
    }

    udec256!(0)
}

pub async fn get_tracked_liquidity_usd(
    token_amount0: UD256,
    token0: &DatabaseToken,
    token_amount1: UD256,
    token1: &DatabaseToken,
    db: &Database,
) -> UD256 {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.mul(bundle.eth_price);
    let price1 = token1.derived_eth.mul(bundle.eth_price);

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0.mul(price0).add(token_amount1.mul(price1));
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0.mul(price0).mul(udec256!(2));
    }

    if !WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount1.mul(price1).mul(udec256!(2));
    }

    udec256!(0)
}

pub async fn update_factory_day_data(
    db: &Database,
    timestamp: i64,
) -> DatabaseFactoryDayData {
    let factory = db.get_factory().await;
    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;

    let mut factory_day_data =
        match db.get_factory_day_data(&day_id.to_string()).await {
            Some(factory_day_data) => factory_day_data,
            None => DatabaseFactoryDayData::new(
                day_id.to_string(),
                day_start_timestamp,
            ),
        };

    factory_day_data.total_liquidity_usd = factory.total_liquidity_usd;
    factory_day_data.total_liquidity_eth = factory.total_liquidity_eth;
    factory_day_data.tx_count = factory.tx_count;

    db.update_factory_day_data(&factory_day_data).await;

    factory_day_data
}

pub async fn update_pair_day_data(
    log: &Log,
    timestamp: i64,
    db: &Database,
) -> DatabasePairDayData {
    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;
    let day_pair_id =
        format!("{}-{}", log.address().to_string().to_lowercase(), day_id); // TODO: check if this is correct;

    let pair = db
        .get_pair(&log.address().to_string().to_lowercase())
        .await
        .unwrap();

    let mut pair_day_data =
        match db.get_pair_day_data(&day_pair_id.to_string()).await {
            Some(pair_day_data) => pair_day_data,
            None => DatabasePairDayData::new(
                day_pair_id,
                day_start_timestamp,
                pair.id,
                pair.token0,
                pair.token1,
            ),
        };

    pair_day_data.total_supply = pair.total_supply;
    pair_day_data.reserve0 = pair.reserve0;
    pair_day_data.reserve1 = pair.reserve1;
    pair_day_data.reserve_usd = pair.reserve_usd;
    pair_day_data.daily_txns += 1;

    db.update_pair_day_data(&pair_day_data).await;

    pair_day_data
}

pub async fn update_pair_hour_data(
    log: &Log,
    timestamp: i64,
    db: &Database,
) -> DatabasePairHourData {
    let hour_index = timestamp / 3600;
    let hour_start_unix = hour_index * 3600;
    let hour_pair_id = format!(
        "{}-{}",
        log.address().to_string().to_lowercase(),
        hour_index
    ); // TODO: check if this is correct;

    let pair = db
        .get_pair(&log.address().to_string().to_lowercase())
        .await
        .unwrap();

    let mut pair_hour_data =
        match db.get_pair_hour_data(&hour_pair_id.to_string()).await {
            Some(pair_hour_data) => pair_hour_data,
            None => DatabasePairHourData::new(
                hour_pair_id,
                hour_start_unix,
                pair.id,
            ),
        };

    pair_hour_data.total_supply = pair.total_supply;
    pair_hour_data.reserve0 = pair.reserve0;
    pair_hour_data.reserve1 = pair.reserve1;
    pair_hour_data.reserve_usd = pair.reserve_usd;
    pair_hour_data.hourly_txns += 1;

    db.update_pair_hour_data(&pair_hour_data).await;

    pair_hour_data
}

pub async fn update_token_day_data(
    token: &DatabaseToken,
    timestamp: i64,
    db: &Database,
) -> DatabaseTokenDayData {
    let bundle = db.get_bundle().await;
    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;
    let token_day_id = format!("{}-{}", token.id, day_id);

    let mut token_day_data =
        match db.get_token_day_data(&token_day_id.to_string()).await {
            Some(token_day_data) => token_day_data,
            None => DatabaseTokenDayData::new(
                token_day_id,
                day_start_timestamp,
                token.id.clone(),
                token.derived_eth.mul(bundle.eth_price),
            ),
        };

    token_day_data.price_usd = token.derived_eth.mul(bundle.eth_price);
    token_day_data.total_liquidity_token = token.total_liquidity;
    token_day_data.total_liquidity_eth =
        token.total_liquidity.mul(token.derived_eth);
    token_day_data.total_liquidity_usd =
        token_day_data.total_liquidity_eth.mul(bundle.eth_price);
    token_day_data.daily_txns += 1;

    db.update_token_day_data(&token_day_data).await;

    token_day_data
}

fn exponent_to_bigdecimal(decimals: &U256) -> UD256 {
    let mut bd = u256!(1);

    let mut i = u256!(0);
    while i < *decimals {
        bd = bd.mul(u256!(10));
        i += &u256!(1);
    }

    UD256::from_str(&bd.to_string(), Context::default()).unwrap()
}

pub fn convert_token_to_decimal(
    token_amount: &U256,
    exchange_decimals: &U256,
) -> UD256 {
    let token_amount_decimal =
        UD256::from_str(&token_amount.to_string(), Context::default())
            .unwrap();

    if exchange_decimals.is_zero() {
        token_amount_decimal
    } else {
        let divisor = exponent_to_bigdecimal(exchange_decimals);
        token_amount_decimal / divisor
    }
}
