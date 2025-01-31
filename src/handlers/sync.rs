use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use fastnum::udec256;

use crate::{
    configs::Config, db::Database, rpc::Rpc, utils::format::parse_uint112,
};

use super::utils::{
    convert_token_to_decimal, find_eth_per_token, get_eth_price_usd,
    get_tracked_liquidity_usd,
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

    // Get the pair
    let mut pair = db.get_pair(event.address.to_string()).await.unwrap();

    // Get the token0
    let mut token0 = db.get_token(pair.token0.clone()).await.unwrap();
    // Get the token1
    let mut token1 = db.get_token(pair.token1.clone()).await.unwrap();

    // Load the factory
    let mut factory = db.get_factory().await;

    factory.total_liquidity_eth =
        factory.total_liquidity_eth.min(pair.tracked_reserve_eth);

    token0.total_liquidity = token0.total_liquidity.min(pair.reserve0);
    token1.total_liquidity = token1.total_liquidity.min(pair.reserve1);

    pair.reserve0 = convert_token_to_decimal(
        &parse_uint112(event.reserve0),
        &token0.decimals,
    );

    pair.reserve1 = convert_token_to_decimal(
        &parse_uint112(event.reserve1),
        &token1.decimals,
    );

    if pair.reserve0.ne(&udec256!(0)) {
        pair.token0_price = pair.reserve0.div(pair.reserve1)
    } else {
        pair.token0_price = udec256!(0)
    }

    if pair.reserve1.ne(&udec256!(0)) {
        pair.token1_price = pair.reserve1.div(pair.reserve0)
    } else {
        pair.token1_price = udec256!(0)
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

    let mut tracked_liquidity_eth = udec256!(0);

    if bundle.eth_price.ne(&udec256!(0)) {
        tracked_liquidity_eth = get_tracked_liquidity_usd(
            pair.reserve0,
            &token0,
            pair.reserve1,
            &token1,
            db,
        )
        .await
    }

    pair.tracked_reserve_eth = tracked_liquidity_eth;
    pair.reserve_eth = pair
        .reserve0
        .mul(token0.derived_eth)
        .add(pair.reserve1.mul(token1.derived_eth));

    pair.reserve_usd = pair.reserve_eth.mul(bundle.eth_price);

    factory.total_liquidity_eth =
        factory.total_liquidity_eth.add(tracked_liquidity_eth);

    factory.total_liquidity_usd =
        factory.total_liquidity_eth.mul(bundle.eth_price);

    token0.total_liquidity = token0.total_liquidity.add(pair.reserve0);

    token1.total_liquidity = token1.total_liquidity.mul(pair.reserve1);

    db.update_pair(&pair).await;
    db.update_factory(&factory).await;
    db.update_token(&token0).await;
    db.update_token(&token1).await;
}
