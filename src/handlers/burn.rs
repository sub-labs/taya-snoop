use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{
    db::Database,
    utils::format::{convert_token_to_decimal, parse_u256},
};

use super::utils::{
    update_dex_day_data, update_pair_day_data, update_pair_hour_data,
    update_token_day_data,
};

sol! {
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
}

pub async fn handle_burn(log: Log, timestamp: i32, db: &Database) {
    let event = Burn::decode_log(&log.inner, true).unwrap();

    let sender_address = event.sender.to_string().to_lowercase();

    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let transaction = db.get_transaction(&transaction_hash).await;
    if transaction.is_none() {
        return;
    }

    let transaction = transaction.unwrap();

    let burns = transaction.burns.clone();

    let burn = burns.last().unwrap().as_ref().unwrap();

    let burn = db.get_burn(burn).await;
    if burn.is_none() {
        return;
    }

    let pair_address = event.address.to_string().to_lowercase();

    let mut pair = db.get_pair(&pair_address).await.unwrap();

    let mut factory = db.get_factory().await;

    let token0 = db.get_token(&pair.token0).await;
    let token1 = db.get_token(&pair.token1).await;

    if token0.is_none() || token1.is_none() {
        return;
    }

    let mut token0 = token0.unwrap();
    let mut token1 = token1.unwrap();

    let token0_amount = convert_token_to_decimal(
        &parse_u256(event.amount0),
        token0.decimals,
    );

    let token1_amount = convert_token_to_decimal(
        &parse_u256(event.amount1),
        token1.decimals,
    );

    token0.tx_count += 1;
    token1.tx_count += 1;

    let bundle = db.get_bundle().await;

    let amount_total_usd = (token1.derived_eth.clone()
        * token1_amount.clone())
        + (token0.derived_eth.clone() * token0_amount.clone())
            * bundle.eth_price;

    pair.tx_count += 1;
    factory.tx_count += 1;

    let mut burn = burn.unwrap();
    burn.sender = sender_address;
    burn.amount0 = token0_amount;
    burn.amount1 = token1_amount;
    burn.log_index = log.log_index.unwrap() as i32;
    burn.amount_usd = amount_total_usd;

    tokio::join!(
        db.update_token(&token0),
        db.update_token(&token1),
        db.update_pair(&pair),
        db.update_factory(&factory),
        db.update_burn(&burn),
        update_pair_day_data(&pair, timestamp, db),
        update_pair_hour_data(&pair, timestamp, db),
        update_dex_day_data(&factory, db, timestamp),
        update_token_day_data(&token0, timestamp, db),
        update_token_day_data(&token1, timestamp, db),
    );
}
