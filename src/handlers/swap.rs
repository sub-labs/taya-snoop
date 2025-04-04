use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use bigdecimal::BigDecimal;

use crate::{
    configs::Config,
    db::{
        models::{
            bundle::DatabaseBundle, factory::DatabaseFactory,
            swap::DatabaseSwap, transaction::DatabaseTransaction,
        },
        Database,
    },
    utils::format::{convert_token_to_decimal, parse_u256, zero_bd},
};

use super::utils::{
    get_tracked_volume_usd, update_dex_day_data, update_pair_day_data,
    update_pair_hour_data, update_token_day_data,
};

sol! {
    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
}

pub async fn handle_swap(
    log: Log,
    block_timestamp: i32,
    db: &Database,
    config: &Config,
    factory: &mut DatabaseFactory,
    bundle: &DatabaseBundle,
) {
    let event = Swap::decode_log(&log.inner, true).unwrap();
    let pair_address = event.address.to_string().to_lowercase();
    let sender_address = event.sender.to_string().to_lowercase();
    let to_address = event.to.to_string().to_lowercase();
    let transaction_hash = log.transaction_hash.unwrap().to_string();
    let block_number = log.block_number.unwrap() as i32;

    let (pair_result, transction_result) = tokio::join!(
        db.get_pair(&pair_address),
        db.get_transaction(&transaction_hash)
    );

    let mut pair = pair_result.unwrap();

    let mut transaction = match transction_result {
        Some(tx) => tx,
        None => DatabaseTransaction::new(
            transaction_hash.clone(),
            block_number,
            block_timestamp,
        ),
    };

    let (token0_result, token1_result) = tokio::join!(
        db.get_token(&pair.token0),
        db.get_token(&pair.token1)
    );

    if token0_result.is_none() || token1_result.is_none() {
        return;
    }

    let mut token0 = token0_result.unwrap();
    let mut token1 = token1_result.unwrap();

    let amount0_in = convert_token_to_decimal(
        &parse_u256(event.amount0In),
        token0.decimals,
    );
    let amount1_in = convert_token_to_decimal(
        &parse_u256(event.amount1In),
        token1.decimals,
    );
    let amount0_out = convert_token_to_decimal(
        &parse_u256(event.amount0Out),
        token0.decimals,
    );
    let amount1_out = convert_token_to_decimal(
        &parse_u256(event.amount1Out),
        token1.decimals,
    );
    let amount0_total = amount0_out.clone() + amount0_in.clone();
    let amount1_total = amount1_out.clone() + amount1_in.clone();

    let derived_amount_eth: BigDecimal = ((token1.derived_eth.clone()
        * (amount1_total.clone()))
        + (token0.derived_eth.clone() * amount0_total.clone()))
        / 2;

    let derived_amount_usd = derived_amount_eth * bundle.eth_price.clone();

    let tracked_amount_usd = get_tracked_volume_usd(
        amount0_total.clone(),
        &token0,
        amount1_total.clone(),
        &token1,
        &pair,
        db,
        config,
    )
    .await;

    let tracked_amount_eth: BigDecimal =
        match bundle.eth_price == zero_bd() {
            true => zero_bd(),
            false => tracked_amount_usd.clone() / bundle.eth_price.clone(),
        };

    token0.trade_volume += amount0_in.clone() + amount0_out.clone();
    token0.trade_volume_usd += tracked_amount_usd.clone();
    token0.untracked_volume_usd += derived_amount_usd.clone();

    token1.trade_volume += amount1_in.clone() + amount1_out.clone();
    token1.trade_volume_usd += tracked_amount_usd.clone();
    token1.untracked_volume_usd += derived_amount_usd.clone();

    token0.tx_count += 1;
    token1.tx_count += 1;

    pair.volume_usd += tracked_amount_usd.clone();
    pair.volume_token0 += amount0_total.clone();
    pair.volume_token1 += amount1_total.clone();
    pair.untracked_volume_usd += derived_amount_usd.clone();
    pair.tx_count += 1;

    factory.total_volume_usd += tracked_amount_usd.clone();
    factory.total_volume_eth =
        factory.total_volume_eth.clone() + tracked_amount_eth.clone();

    factory.untracked_volume_usd += derived_amount_usd.clone();

    factory.tx_count += 1;

    let swap_id = format!(
        "{}-{}",
        transaction_hash.clone(),
        transaction.swaps.len()
    );

    let amount_usd = match tracked_amount_usd == zero_bd() {
        true => derived_amount_usd.clone(),
        false => tracked_amount_usd.clone(),
    };

    let swap = DatabaseSwap::new(
        swap_id,
        log.transaction_hash.unwrap().to_string(),
        block_timestamp,
        pair_address,
        sender_address.clone(),
        sender_address,
        amount0_in,
        amount1_in,
        amount0_out,
        amount1_out,
        log.log_index.unwrap() as i32,
        amount_usd,
        to_address,
    );

    transaction.swaps.push(Some(swap.id.clone()));

    tokio::join!(
        db.update_pair(&pair),
        db.update_token(&token0),
        db.update_token(&token1),
        db.update_swap(&swap),
        db.update_transaction(&transaction),
    );

    let (
        mut pair_day_data,
        mut pair_hour_data,
        mut dex_day_data,
        mut token0_day_data,
        mut token1_day_data,
    ) = tokio::join!(
        update_pair_day_data(&pair, block_timestamp, db),
        update_pair_hour_data(&pair, block_timestamp, db),
        update_dex_day_data(factory, db, block_timestamp),
        update_token_day_data(&token0, block_timestamp, db),
        update_token_day_data(&token1, block_timestamp, db)
    );

    dex_day_data.daily_volume_usd += tracked_amount_usd.clone();
    dex_day_data.daily_volume_eth += tracked_amount_eth;
    dex_day_data.daily_volume_untracked += derived_amount_usd;

    pair_day_data.daily_volume_token0 += amount0_total.clone();
    pair_day_data.daily_volume_token1 += amount1_total.clone();
    pair_day_data.daily_volume_usd += tracked_amount_usd.clone();

    pair_hour_data.hourly_volume_token0 += amount0_total.clone();
    pair_hour_data.hourly_volume_token1 += amount1_total.clone();
    pair_hour_data.hourly_volume_usd += tracked_amount_usd.clone();

    token0_day_data.daily_volume_token += amount0_total.clone();
    token0_day_data.daily_volume_eth +=
        amount0_total.clone() * token0.derived_eth.clone();
    token0_day_data.daily_volume_usd +=
        amount0_total * token0.derived_eth * bundle.eth_price.clone();

    token1_day_data.daily_volume_token += amount1_total.clone();

    token1_day_data.daily_volume_eth +=
        amount1_total.clone() * token1.derived_eth.clone();

    token1_day_data.daily_volume_usd +=
        amount1_total * token1.derived_eth * bundle.eth_price.clone();

    tokio::join!(
        db.update_dex_day_data(&dex_day_data),
        db.update_pair_day_data(&pair_day_data),
        db.update_pair_hour_data(&pair_hour_data),
        db.update_token_day_data(&token0_day_data),
        db.update_token_day_data(&token1_day_data),
    );
}
