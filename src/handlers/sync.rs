use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{
    configs::Config,
    db::{Database, StorageCache},
    utils::format::{convert_token_to_decimal, parse_u112, zero_bd},
};

use super::utils::{
    find_eth_per_token, get_eth_price_usd, get_tracked_liquidity_usd,
};

sol! {
    event Sync(uint112 reserve0, uint112 reserve1);
}

pub async fn handle_sync(
    log: Log,
    db: &Database,
    config: &Config,
    cache: &mut StorageCache,
) {
    let event = Sync::decode_log(&log.inner, true).unwrap();

    let pair_address = event.address.to_string().to_lowercase();

    let mut pair = match cache.pairs.get(&pair_address) {
        Some(pair) => pair.to_owned(),
        None => db.get_pair(&pair_address).await.unwrap(),
    };

    let token0_address = pair.token0.to_lowercase();
    let token1_address = pair.token1.to_lowercase();

    let mut token0 = match cache.tokens.get(&token0_address) {
        Some(token) => token.to_owned(),
        None => match db.get_token(&token0_address).await {
            Some(token) => token,
            None => return,
        },
    };

    let mut token1 = match cache.tokens.get(&token1_address) {
        Some(token) => token.to_owned(),
        None => match db.get_token(&token1_address).await {
            Some(token) => token,
            None => return,
        },
    };

    cache.factory.total_liquidity_eth -= pair.tracked_reserve_eth.clone();

    token0.total_liquidity -= pair.reserve0;
    token1.total_liquidity -= pair.reserve1;

    pair.reserve0 = convert_token_to_decimal(
        &parse_u112(event.reserve0),
        token0.decimals,
    );

    pair.reserve1 = convert_token_to_decimal(
        &parse_u112(event.reserve1),
        token1.decimals,
    );

    if pair.reserve1 != zero_bd() {
        pair.token0_price = pair.reserve0.clone() / pair.reserve1.clone()
    } else {
        pair.token0_price = zero_bd()
    }

    if pair.reserve0 != zero_bd() {
        pair.token1_price = pair.reserve1.clone() / pair.reserve0.clone()
    } else {
        pair.token1_price = zero_bd()
    }

    cache.bundle.eth_price = get_eth_price_usd(db, config, cache).await;

    token0.derived_eth =
        find_eth_per_token(&token0, db, config, cache).await;

    token1.derived_eth =
        find_eth_per_token(&token1, db, config, cache).await;

    let mut tracked_liquidity_eth = zero_bd();

    if cache.bundle.eth_price != zero_bd() {
        tracked_liquidity_eth = get_tracked_liquidity_usd(
            pair.reserve0.clone(),
            &token0,
            pair.reserve1.clone(),
            &token1,
            db,
            config,
        )
        .await
            / cache.bundle.eth_price.clone()
    }

    pair.tracked_reserve_eth = tracked_liquidity_eth.clone();
    pair.reserve_eth = (pair.reserve0.clone()
        * token0.derived_eth.clone())
        + (pair.reserve1.clone() * token1.derived_eth.clone());

    pair.reserve_usd =
        pair.reserve_eth.clone() * cache.bundle.eth_price.clone();

    cache.factory.total_liquidity_eth += tracked_liquidity_eth;

    cache.factory.total_liquidity_usd =
        cache.factory.total_liquidity_eth.clone()
            * cache.bundle.eth_price.clone();

    token0.total_liquidity += pair.reserve0.clone();
    token1.total_liquidity += pair.reserve1.clone();

    db.update_pair(&pair).await
}
