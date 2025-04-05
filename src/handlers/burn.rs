use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{
    db::{Database, StorageCache},
    utils::format::{convert_token_to_decimal, parse_u256},
};

use super::utils::{
    update_dex_day_data, update_pair_day_data, update_pair_hour_data,
    update_token_day_data,
};

sol! {
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
}

pub async fn handle_burn(
    log: Log,
    timestamp: i32,
    db: &Database,
    cache: &mut StorageCache,
) {
    let event = Burn::decode_log(&log.inner, true).unwrap();
    let sender_address = event.sender.to_string().to_lowercase();
    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let transaction = match cache.transactions.get(&transaction_hash) {
        Some(transaction) => transaction.to_owned(),
        None => match db.get_transaction(&transaction_hash).await {
            Some(tx) => tx,
            None => return,
        },
    };

    let burns = transaction.burns.clone();
    let burn_id = burns.last().unwrap().as_ref().unwrap();

    let mut burn = match cache.burns.get(burn_id) {
        Some(transaction) => transaction.to_owned(),
        None => match db.get_burn(burn_id).await {
            Some(burn) => burn,
            None => return,
        },
    };

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
    let amount_total_usd = (token1.derived_eth.clone()
        * token1_amount.clone())
        + (token0.derived_eth.clone() * token0_amount.clone())
            * cache.bundle.eth_price.clone();

    pair.tx_count += 1;
    cache.factory.tx_count += 1;

    burn.sender = sender_address;
    burn.amount0 = token0_amount;
    burn.amount1 = token1_amount;
    burn.log_index = log.log_index.unwrap() as i32;
    burn.amount_usd = amount_total_usd;

    cache.burns.insert(burn.id.clone(), burn);
    cache.tokens.insert(token0.id.clone(), token0.clone());
    cache.tokens.insert(token1.id.clone(), token1.clone());

    tokio::join!(
        update_pair_day_data(&pair, timestamp, db),
        update_pair_hour_data(&pair, timestamp, db),
        update_dex_day_data(db, timestamp, cache),
        update_token_day_data(&token0, timestamp, db),
        update_token_day_data(&token1, timestamp, db),
    );
}
