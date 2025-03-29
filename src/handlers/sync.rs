use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{
    configs::Config,
    db::Database,
    rpc::Rpc,
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
    rpc: &Rpc,
    config: &Config,
) {
    let event = Sync::decode_log(&log.inner, true).unwrap();

    let pair_address = event.address.to_string().to_lowercase();

    // Get the pair
    let mut pair = db.get_pair(&pair_address).await.unwrap();

    // Get the token0
    let mut token0 = db.get_token(&pair.token0).await.unwrap();
    // Get the token1
    let mut token1 = db.get_token(&pair.token1).await.unwrap();

    // Load the factory
    let mut factory = db.get_factory().await;

    factory.total_liquidity_eth -= pair.tracked_reserve_eth.clone();

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

    db.update_pair(&pair).await;

    let mut bundle = db.get_bundle().await;

    bundle.eth_price = get_eth_price_usd(db, config).await;

    db.update_bundle(&bundle).await;

    token0.derived_eth =
        find_eth_per_token(&token0, rpc, db, config).await;

    token1.derived_eth =
        find_eth_per_token(&token1, rpc, db, config).await;

    db.update_token(&token0).await;
    db.update_token(&token1).await;

    let mut tracked_liquidity_eth = zero_bd();

    if bundle.eth_price != zero_bd() {
        tracked_liquidity_eth = get_tracked_liquidity_usd(
            pair.reserve0.clone(),
            &token0,
            pair.reserve1.clone(),
            &token1,
            db,
            config,
        )
        .await
            / bundle.eth_price.clone()
    }

    pair.tracked_reserve_eth = tracked_liquidity_eth.clone();
    pair.reserve_eth = (pair.reserve0.clone()
        * token0.derived_eth.clone())
        + (pair.reserve1.clone() * token1.derived_eth.clone());

    pair.reserve_usd = pair.reserve_eth.clone() * bundle.eth_price.clone();

    factory.total_liquidity_eth += tracked_liquidity_eth;

    factory.total_liquidity_usd =
        factory.total_liquidity_eth.clone() * bundle.eth_price;

    token0.total_liquidity += pair.reserve0.clone();
    token1.total_liquidity += pair.reserve1.clone();

    db.update_pair(&pair).await;
    db.update_factory(&factory).await;
    db.update_token(&token0).await;
    db.update_token(&token1).await;
}
