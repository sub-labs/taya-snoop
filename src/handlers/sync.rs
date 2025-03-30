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

    let (pair_result, mut factory, mut bundle) = tokio::join!(
        db.get_pair(&pair_address),
        db.get_factory(),
        db.get_bundle()
    );

    let mut pair = pair_result.unwrap();

    let (token0_result, token1_result) = tokio::join!(
        db.get_token(&pair.token0),
        db.get_token(&pair.token1)
    );

    let mut token0 = token0_result.unwrap();
    let mut token1 = token1_result.unwrap();

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

    // IMPORTANT:
    // Update the pair before checking prices to prevent zero division if the pair updated is used to calculate prices
    db.update_pair(&pair).await;

    bundle.eth_price = get_eth_price_usd(db, config).await;

    token0.derived_eth =
        find_eth_per_token(&token0, rpc, db, config).await;

    token1.derived_eth =
        find_eth_per_token(&token1, rpc, db, config).await;

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
        factory.total_liquidity_eth.clone() * bundle.eth_price.clone();

    token0.total_liquidity += pair.reserve0.clone();
    token1.total_liquidity += pair.reserve1.clone();

    tokio::join!(
        db.update_bundle(&bundle),
        db.update_pair(&pair),
        db.update_factory(&factory),
        db.update_token(&token0),
        db.update_token(&token1),
    );
}
