use alloy::{
    primitives::{utils::format_units, U256},
    rpc::types::Log,
    sol,
    sol_types::SolEvent,
};
use bigdecimal::{BigDecimal, FromPrimitive, Zero};

use crate::{configs::Config, db::Database, rpc::Rpc};

use super::utils::{
    find_eth_per_token, get_eth_price_usd, get_tracked_liquidity_usd,
};

sol! {
    event Sync(uint112 reserve0, uint112 reserve1);
}

pub async fn handle_syncs(
    log: Log,
    db: &Database,
    rpc: &Rpc,
    config: &Config,
) {
    let event = Sync::decode_log(&log.inner, true).unwrap();

    // Get the pair
    let mut pair = db.get_pair(event.address.to_string()).await.unwrap();

    // Get the token0
    let mut token0 = db.get_token(pair.token0.clone()).await.unwrap();
    // Get the token1
    let mut token1 = db.get_token(pair.token1.clone()).await.unwrap();

    // Load the factory
    let mut factory = db.get_factory().await;

    factory.total_liquidity_eth -= pair.tracked_reserve_eth.clone();

    token0.total_liquidity -= pair.reserve0;
    token1.total_liquidity -= pair.reserve1;

    pair.reserve0 = BigDecimal::from_f64(
        format_units(U256::from(event.reserve0), token0.decimals as u8)
            .unwrap()
            .parse::<f64>()
            .unwrap(),
    )
    .unwrap();

    pair.reserve1 = BigDecimal::from_f64(
        format_units(U256::from(event.reserve1), token0.decimals as u8)
            .unwrap()
            .parse::<f64>()
            .unwrap(),
    )
    .unwrap();

    if pair.reserve0.ne(&BigDecimal::zero()) {
        pair.token0_price = pair.reserve0.clone() / pair.reserve1.clone()
    } else {
        pair.token0_price = BigDecimal::zero()
    }

    if pair.reserve1.ne(&BigDecimal::zero()) {
        pair.token1_price = pair.reserve1.clone() / pair.reserve0.clone()
    } else {
        pair.token1_price = BigDecimal::zero()
    }

    db.update_pair(&pair).await;

    let mut bundle = db.get_bundle().await;

    bundle.eth_price = get_eth_price_usd(db).await;

    db.update_bundle(&bundle).await;

    token0.derived_eth =
        find_eth_per_token(&token0, rpc, db, config).await;

    token1.derived_eth =
        find_eth_per_token(&token1, rpc, db, config).await;

    db.update_token(&token0).await;
    db.update_token(&token1).await;

    let mut tracked_liquidity_eth = BigDecimal::zero();
    if bundle.eth_price.ne(&BigDecimal::zero()) {
        tracked_liquidity_eth = get_tracked_liquidity_usd(
            pair.reserve0.clone(),
            &token0,
            pair.reserve1.clone(),
            &token1,
            db,
        )
        .await
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
