use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{
    db::{
        models::{bundle::DatabaseBundle, factory::DatabaseFactory},
        Database,
    },
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
    factory: &mut DatabaseFactory,
    bundle: &DatabaseBundle,
) {
    let event = Burn::decode_log(&log.inner, true).unwrap();
    let sender_address = event.sender.to_string().to_lowercase();
    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let transaction = match db.get_transaction(&transaction_hash).await {
        Some(tx) => tx,
        None => return,
    };

    let burns = transaction.burns.clone();
    let burn_id = burns.last().unwrap().as_ref().unwrap();
    let mut burn = match db.get_burn(burn_id).await {
        Some(burn) => burn,
        None => return,
    };

    let pair_address = event.address.to_string().to_lowercase();

    let mut pair = db.get_pair(&pair_address).await.unwrap();

    let (token0_request, token1_request) = tokio::join!(
        db.get_token(&pair.token0),
        db.get_token(&pair.token1)
    );
    let mut token0 = match token0_request {
        Some(token) => token,
        None => return,
    };
    let mut token1 = match token1_request {
        Some(token) => token,
        None => return,
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
            * bundle.eth_price.clone();

    pair.tx_count += 1;
    factory.tx_count += 1;

    burn.sender = sender_address;
    burn.amount0 = token0_amount;
    burn.amount1 = token1_amount;
    burn.log_index = log.log_index.unwrap() as i32;
    burn.amount_usd = amount_total_usd;

    tokio::join!(
        db.update_token(&token0),
        db.update_token(&token1),
        db.update_burn(&burn),
        update_pair_day_data(&pair, timestamp, db),
        update_pair_hour_data(&pair, timestamp, db),
        update_dex_day_data(factory, db, timestamp),
        update_token_day_data(&token0, timestamp, db),
        update_token_day_data(&token1, timestamp, db),
    );
}
