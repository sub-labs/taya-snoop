use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use fastnum::UD256;

use crate::{db::Database, utils::format::parse_ud256};

use super::utils::{
    convert_token_to_decimal, update_factory_day_data,
    update_pair_day_data, update_pair_hour_data, update_token_day_data,
};

sol! {
    event Mint(address indexed sender, uint amount0, uint amount1);
}

pub async fn handle_mint(log: Log, timestamp: i64, db: &Database) {
    let event = Mint::decode_log(&log.inner, true).unwrap();

    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let transaction = db.get_transaction(&transaction_hash).await;
    if transaction.is_none() {
        return;
    }

    let transaction = transaction.unwrap();

    let mints = transaction.mints.clone();
    let mint = db.get_mint(mints.last().unwrap()).await;
    if mint.is_none() {
        return;
    }

    let mut pair = db.get_pair(&event.address.to_string()).await.unwrap();

    let mut factory = db.get_factory().await;

    let token0 = db.get_token(&pair.token0).await;
    let token1 = db.get_token(&pair.token1).await;

    if token0.is_none() || token1.is_none() {
        return;
    }

    let mut token0 = token0.unwrap();
    let mut token1 = token1.unwrap();

    let token0_amount = convert_token_to_decimal(
        &parse_ud256(event.amount0),
        &UD256::from(token0.decimals),
    );

    let token1_amount = convert_token_to_decimal(
        &parse_ud256(event.amount1),
        &UD256::from(token1.decimals),
    );

    token0.tx_count += 1;
    token1.tx_count += 1;

    let bundle = db.get_bundle().await;

    let amount_total_usd = token1
        .derived_eth
        .mul(token1_amount)
        .add(token0.derived_eth.mul(bundle.eth_price));

    pair.tx_count += 1;
    factory.tx_count += 1;

    db.update_token(&token0).await;
    db.update_token(&token1).await;
    db.update_pair(&pair).await;
    db.update_factory(&factory).await;

    let mut mint = mint.unwrap();
    mint.sender = event.sender.to_string().to_lowercase();
    mint.amount0 = token0_amount;
    mint.amount1 = token1_amount;
    mint.log_index = log.log_index.unwrap() as i64;
    mint.amount_usd = amount_total_usd;

    db.update_mint(&mint).await;

    update_pair_day_data(&log, timestamp, db).await;
    update_pair_hour_data(&log, timestamp, db).await;
    update_factory_day_data(db, timestamp).await;
    update_token_day_data(&token0, timestamp, db).await;
    update_token_day_data(&token1, timestamp, db).await;
}
