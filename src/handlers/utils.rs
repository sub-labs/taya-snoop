use crate::{
    configs::Config,
    db::{
        models::{
            data::{
                DatabaseDexDayData, DatabasePairDayData,
                DatabasePairHourData, DatabaseTokenDayData,
            },
            pair::DatabasePair,
            token::DatabaseToken,
        },
        Database, StorageCache,
    },
    utils::format::{one_bd, zero_bd},
};
use bigdecimal::BigDecimal;

pub async fn get_eth_price_usd(
    db: &Database,
    config: &Config,
    cache: &StorageCache,
) -> BigDecimal {
    let usdc_pair: Option<DatabasePair> = match config.chain.usdc_weth_pair
    {
        Some(pair) => match cache.pairs.get(pair) {
            Some(pair) => Some(pair.to_owned()),
            None => db.get_pair(pair).await,
        },
        None => None,
    };

    let usdt_pair: Option<DatabasePair> = match config.chain.usdt_weth_pair
    {
        Some(pair) => match cache.pairs.get(pair) {
            Some(pair) => Some(pair.to_owned()),
            None => db.get_pair(pair).await,
        },
        None => None,
    };

    match (usdc_pair, usdt_pair) {
        (Some(usdc), Some(usdt)) => {
            let total_liquidity_eth =
                usdt.reserve0.clone() + usdc.reserve0.clone();

            let usdt_weight = usdt.reserve0 / total_liquidity_eth.clone();

            let usdc_weight = usdc.reserve0 / total_liquidity_eth.clone();

            (usdt.token1_price * usdt_weight)
                + (usdc.token1_price * usdc_weight)
        }
        (Some(usdc), _) => usdc.token1_price,
        // No pairs
        _ => zero_bd(),
    }
}

pub async fn find_eth_per_token(
    token: &DatabaseToken,
    db: &Database,
    config: &Config,
    cache: &StorageCache,
) -> BigDecimal {
    let token_address = token.id.to_lowercase();

    if token_address == *config.chain.weth {
        return one_bd();
    }

    // Loop through a set of whitelisted tokens to check if there is any pair for this token.
    for whitelist_token in config.chain.whitelist_tokens {
        let whitelist_token_address = whitelist_token.to_lowercase();

        let pair = db
            .get_pair_for_tokens(&token_address, &whitelist_token_address)
            .await;

        if pair.is_none() {
            continue;
        }

        if pair.is_none() {
            continue;
        }

        let pair = pair.unwrap();

        let pair_token0_address = pair.token0.to_lowercase();
        let pair_token1_address = pair.token0.to_lowercase();

        if pair_token0_address == token_address
            && pair.reserve_eth
                > BigDecimal::from(
                    config.chain.minimum_liquidity_threshold_eth,
                )
        {
            let token0 = match cache.tokens.get(&pair_token0_address) {
                Some(token) => token.to_owned(),
                None => match db.get_token(&pair_token0_address).await {
                    Some(token) => token,
                    None => continue,
                },
            };

            return pair.token0_price * token0.derived_eth;
        }

        if pair_token1_address == token_address
            && pair.reserve_eth
                > BigDecimal::from(
                    config.chain.minimum_liquidity_threshold_eth,
                )
        {
            let token1 = match cache.tokens.get(&pair_token1_address) {
                Some(token) => token.to_owned(),
                None => match db.get_token(&pair_token1_address).await {
                    Some(token) => token,
                    None => continue,
                },
            };

            return pair.token1_price * token1.derived_eth;
        }
    }

    zero_bd()
}

pub async fn get_tracked_volume_usd(
    token_amount0: BigDecimal,
    token0: &DatabaseToken,
    token_amount1: BigDecimal,
    token1: &DatabaseToken,
    pair: &DatabasePair,
    db: &Database,
    config: &Config,
) -> BigDecimal {
    let bundle = db.get_bundle().await;

    let price0: BigDecimal =
        token0.derived_eth.clone() * bundle.eth_price.clone();
    let price1: BigDecimal =
        token1.derived_eth.clone() * bundle.eth_price.clone();

    let token0_address = token0.id.to_lowercase();
    let token1_address = token1.id.to_lowercase();

    if pair.liquidity_provider_count < 5 {
        let reserve0_usd = pair.reserve0.clone() * price0.clone();
        let reserve1_usd = pair.reserve1.clone() * price1.clone();

        let minimum_usd_threshold =
            BigDecimal::from(config.chain.minimum_usd_threshold_new_pairs);

        if config.chain.whitelist_tokens.contains(&token0_address.as_str())
            && config
                .chain
                .whitelist_tokens
                .contains(&token1_address.as_str())
            && (reserve0_usd.clone() + reserve1_usd.clone())
                < minimum_usd_threshold
        {
            return zero_bd();
        }

        if config.chain.whitelist_tokens.contains(&token0_address.as_str())
            && !config
                .chain
                .whitelist_tokens
                .contains(&token1_address.as_str())
            && (reserve0_usd.clone() * 2) < minimum_usd_threshold
        {
            return zero_bd();
        }

        if !config
            .chain
            .whitelist_tokens
            .contains(&token0_address.as_str())
            && config
                .chain
                .whitelist_tokens
                .contains(&token1_address.as_str())
            && (reserve1_usd * 2) < minimum_usd_threshold
        {
            return zero_bd();
        }
    }

    if config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && config.chain.whitelist_tokens.contains(&token1_address.as_str())
    {
        return ((token_amount0 * price0) + (token_amount1 * price1)) / 2;
    }

    if config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && !config
            .chain
            .whitelist_tokens
            .contains(&token1_address.as_str())
    {
        return token_amount0 * price0;
    }

    if !config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && config.chain.whitelist_tokens.contains(&token1_address.as_str())
    {
        return token_amount1 * price1;
    }

    zero_bd()
}

pub async fn get_tracked_liquidity_usd(
    token_amount0: BigDecimal,
    token0: &DatabaseToken,
    token_amount1: BigDecimal,
    token1: &DatabaseToken,
    db: &Database,
    config: &Config,
) -> BigDecimal {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.clone() * bundle.eth_price.clone();
    let price1 = token1.derived_eth.clone() * bundle.eth_price.clone();

    let token0_address = token0.id.to_lowercase();
    let token1_address = token1.id.to_lowercase();

    if config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && config.chain.whitelist_tokens.contains(&token1_address.as_str())
    {
        return (token_amount0 * price0) + (token_amount1 * price1);
    }

    if config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && !config
            .chain
            .whitelist_tokens
            .contains(&token1_address.as_str())
    {
        return (token_amount0 * price0) * 2;
    }

    if !config.chain.whitelist_tokens.contains(&token0_address.as_str())
        && !config
            .chain
            .whitelist_tokens
            .contains(&token1_address.as_str())
    {
        return (token_amount1 * price1) * 2;
    }

    zero_bd()
}

pub async fn update_dex_day_data(
    db: &Database,
    timestamp: i32,
    cache: &mut StorageCache,
) -> DatabaseDexDayData {
    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;

    let mut factory_day_data =
        match cache.dex_day_data.get(&day_id.to_string()) {
            Some(factory_day_data) => factory_day_data.to_owned(),
            None => match db.get_dex_day_data(&day_id.to_string()).await {
                Some(factory_day_data) => factory_day_data,
                None => DatabaseDexDayData::new(
                    day_id.to_string(),
                    day_start_timestamp,
                ),
            },
        };

    factory_day_data.total_liquidity_usd =
        cache.factory.total_liquidity_usd.clone();
    factory_day_data.total_liquidity_eth =
        cache.factory.total_liquidity_eth.clone();
    factory_day_data.tx_count = cache.factory.tx_count;

    cache
        .dex_day_data
        .insert(factory_day_data.id.clone(), factory_day_data.clone());

    factory_day_data
}

pub async fn update_pair_day_data(
    pair: &DatabasePair,
    timestamp: i32,
    db: &Database,
    cache: &mut StorageCache,
) -> DatabasePairDayData {
    let pair_address = pair.id.to_lowercase();

    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;
    let day_pair_id = format!("{}-{}", pair_address, day_id);

    let token0_address = pair.token0.to_lowercase();
    let token1_address = pair.token1.to_lowercase();

    let mut pair_day_data = match cache.pairs_day_data.get(&day_pair_id) {
        Some(pair_day_data) => pair_day_data.to_owned(),
        None => match db.get_pair_day_data(&day_pair_id.to_string()).await
        {
            Some(pair_day_data) => pair_day_data,
            None => DatabasePairDayData::new(
                day_pair_id,
                day_start_timestamp,
                pair_address,
                token0_address,
                token1_address,
            ),
        },
    };

    pair_day_data.total_supply = pair.total_supply.clone();
    pair_day_data.reserve0 = pair.reserve0.clone();
    pair_day_data.reserve1 = pair.reserve1.clone();
    pair_day_data.reserve_usd = pair.reserve_usd.clone();
    pair_day_data.daily_txns += 1;

    cache
        .pairs_day_data
        .insert(pair_day_data.id.clone(), pair_day_data.clone());

    pair_day_data
}

pub async fn update_pair_hour_data(
    pair: &DatabasePair,
    timestamp: i32,
    db: &Database,
    cache: &mut StorageCache,
) -> DatabasePairHourData {
    let pair_address = pair.id.to_lowercase();

    let hour_index = timestamp / 3600;
    let hour_start_unix = hour_index * 3600;
    let hour_pair_id =
        format!("{}-{}", pair_address.to_lowercase(), hour_index);

    let mut pair_hour_data = match cache.pairs_hour_data.get(&hour_pair_id)
    {
        Some(pair_hour_data) => pair_hour_data.to_owned(),
        None => {
            match db.get_pair_hour_data(&hour_pair_id.to_string()).await {
                Some(pair_hour_data) => pair_hour_data,
                None => DatabasePairHourData::new(
                    hour_pair_id,
                    hour_start_unix,
                    pair_address,
                ),
            }
        }
    };

    pair_hour_data.total_supply = pair.total_supply.clone();
    pair_hour_data.reserve0 = pair.reserve0.clone();
    pair_hour_data.reserve1 = pair.reserve1.clone();
    pair_hour_data.reserve_usd = pair.reserve_usd.clone();
    pair_hour_data.hourly_txns += 1;

    cache
        .pairs_hour_data
        .insert(pair_hour_data.id.clone(), pair_hour_data.clone());

    pair_hour_data
}

pub async fn update_token_day_data(
    token: &DatabaseToken,
    timestamp: i32,
    db: &Database,
    cache: &mut StorageCache,
) -> DatabaseTokenDayData {
    let bundle = db.get_bundle().await;
    let day_id = timestamp / 86400;
    let day_start_timestamp = day_id * 86400;

    let token_address = token.id.to_lowercase();

    let token_day_id = format!("{}-{}", token_address, day_id);

    let mut token_day_data = match cache.tokens_day_data.get(&token_day_id)
    {
        Some(token_day_data) => token_day_data.to_owned(),
        None => {
            match db.get_token_day_data(&token_day_id.to_string()).await {
                Some(token_day_data) => token_day_data,
                None => DatabaseTokenDayData::new(
                    token_day_id,
                    day_start_timestamp,
                    token_address.clone(),
                    token.derived_eth.clone() * bundle.eth_price.clone(),
                ),
            }
        }
    };

    token_day_data.price_usd =
        token.derived_eth.clone() * bundle.eth_price.clone();
    token_day_data.total_liquidity_token = token.total_liquidity.clone();
    token_day_data.total_liquidity_eth =
        token.total_liquidity.clone() * token.derived_eth.clone();
    token_day_data.total_liquidity_usd =
        token_day_data.total_liquidity_eth.clone()
            * bundle.eth_price.clone();
    token_day_data.daily_txns += 1;

    cache
        .tokens_day_data
        .insert(token_day_data.id.clone(), token_day_data.clone());

    token_day_data
}
