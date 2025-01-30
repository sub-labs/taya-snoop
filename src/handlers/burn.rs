use alloy::{
    primitives::{utils::format_units, U256},
    rpc::types::Log,
    sol,
    sol_types::SolEvent,
};
use bigdecimal::{BigDecimal, FromPrimitive};

use crate::db::Database;

use super::utils::{
    update_factory_day_data, update_pair_day_data, update_pair_hour_data,
    update_token_day_data,
};

sol! {
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
}

pub async fn handle_burns(log: Log, db: &Database) {
    let event = Burn::decode_log(&log.inner, true).unwrap();

    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let transaction = db.get_transaction(transaction_hash).await;
    if transaction.is_none() {
        return;
    }

    let transaction = transaction.unwrap();

    let burns = transaction.burns.clone();
    let burn = db.get_burn(burns.last().unwrap()).await;
    if burn.is_none() {
        return;
    }

    let mut pair = db.get_pair(event.address.to_string()).await.unwrap();

    let mut factory = db.get_factory().await;

    let token0 = db.get_token(pair.token0.clone()).await;
    let token1 = db.get_token(pair.token1.clone()).await;

    if token0.is_none() || token1.is_none() {
        return;
    }

    let mut token0 = token0.unwrap();
    let mut token1 = token1.unwrap();

    let token0_amount = BigDecimal::from_f64(
        format_units(U256::from(event.amount0), token0.decimals as u8)
            .unwrap()
            .parse::<f64>()
            .unwrap(),
    )
    .unwrap();

    let token1_amount = BigDecimal::from_f64(
        format_units(U256::from(event.amount1), token1.decimals as u8)
            .unwrap()
            .parse::<f64>()
            .unwrap(),
    )
    .unwrap();

    token0.tx_count += 1;
    token1.tx_count += 1;

    let bundle = db.get_bundle().await;

    let amount_total_usd = ((token1.derived_eth.clone()
        * token1_amount.clone())
        + (token0.derived_eth.clone() * token0_amount.clone()))
        * bundle.eth_price;

    pair.tx_count += 1;
    factory.tx_count += 1;

    db.update_token(&token0).await;
    db.update_token(&token1).await;
    db.update_pair(&pair).await;
    db.update_factory(&factory).await;

    let mut burn = burn.unwrap();
    burn.sender = event.sender.to_string().to_lowercase();
    burn.amount0 = token0_amount;
    burn.amount1 = token1_amount;
    burn.log_index = log.log_index.unwrap();
    burn.amount_usd = amount_total_usd;

    db.update_burn(&burn).await;
    update_pair_day_data(&log).await;
    update_pair_hour_data(&log).await;
    update_factory_day_data(&log).await;
    update_token_day_data(&token0, &log).await;
    update_token_day_data(&token1, &log).await;
}
