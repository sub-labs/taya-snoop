use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use fastnum::{u256, U256};

use crate::db::Database;

use super::utils::{
    convert_token_to_decimal, parse_uint256, update_factory_day_data,
    update_pair_day_data, update_pair_hour_data, update_token_day_data,
};

sol! {
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
}

pub async fn handle_burn(log: Log, db: &Database) {
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

    let token0_amount = convert_token_to_decimal(
        &parse_uint256(event.amount0),
        &token0.decimals,
    );

    let token1_amount = convert_token_to_decimal(
        &parse_uint256(event.amount1),
        &token1.decimals,
    );

    token0.tx_count = token0.tx_count.add(u256!(1));
    token1.tx_count = token1.tx_count.add(u256!(1));

    let bundle = db.get_bundle().await;

    let amount_total_usd = token1
        .derived_eth
        .mul(token1_amount)
        .add(token0.derived_eth.mul(bundle.eth_price));

    pair.tx_count = pair.tx_count.add(u256!(1));
    factory.tx_count = factory.tx_count.add(u256!(1));

    db.update_token(&token0).await;
    db.update_token(&token1).await;
    db.update_pair(&pair).await;
    db.update_factory(&factory).await;

    let mut burn = burn.unwrap();
    burn.sender = event.sender.to_string().to_lowercase();
    burn.amount0 = token0_amount;
    burn.amount1 = token1_amount;
    burn.log_index = U256::from(log.log_index.unwrap());
    burn.amount_usd = amount_total_usd;

    db.update_burn(&burn).await;
    update_pair_day_data(&log).await;
    update_pair_hour_data(&log).await;
    update_factory_day_data(&log).await;
    update_token_day_data(&token0, &log).await;
    update_token_day_data(&token1, &log).await;
}
