use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use fastnum::udec256;

use crate::{
    db::{
        models::{
            swap::{DatabaseSwap, SwapAmounts, SwapData},
            transaction::DatabaseTransaction,
        },
        Database,
    },
    utils::format::parse_uint256,
};

use super::utils::{
    convert_token_to_decimal, get_tracked_volume_usd,
    update_factory_day_data, update_pair_day_data, update_pair_hour_data,
    update_token_day_data,
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

pub async fn handle_swap(log: Log, db: &Database) {
    let event = Swap::decode_log(&log.inner, true).unwrap();

    let mut pair = db.get_pair(&event.address.to_string()).await.unwrap();

    let token0 = db.get_token(&pair.token0).await;
    let token1 = db.get_token(&pair.token1).await;

    if token0.is_none() || token1.is_none() {
        return;
    }

    let mut token0 = token0.unwrap();
    let mut token1 = token1.unwrap();

    let amount0_in = convert_token_to_decimal(
        &parse_uint256(event.amount0In),
        &token0.decimals,
    );

    let amount1_in = convert_token_to_decimal(
        &parse_uint256(event.amount1In),
        &token1.decimals,
    );

    let amount0_out = convert_token_to_decimal(
        &parse_uint256(event.amount0Out),
        &token0.decimals,
    );

    let amount1_out = convert_token_to_decimal(
        &parse_uint256(event.amount1Out),
        &token1.decimals,
    );

    let amount0_total = amount0_out.add(amount0_in);
    let amount1_total = amount1_out.add(amount1_in);

    let bundle = db.get_bundle().await;

    let derived_amount_eth = token1
        .derived_eth
        .mul(amount1_total)
        .add(token0.derived_eth.mul(amount0_total))
        .div(udec256!(2));

    let derived_amount_usd = derived_amount_eth.mul(bundle.eth_price);

    let tracked_amount_usd = get_tracked_volume_usd(
        amount0_total,
        &token0,
        amount1_total,
        &token1,
        &pair,
        db,
    )
    .await;

    let tracked_amount_eth = match bundle.eth_price.eq(&udec256!(0)) {
        true => udec256!(0),
        false => tracked_amount_usd.div(bundle.eth_price),
    };

    token0.trade_volume =
        token0.trade_volume.add(amount0_in.add(amount0_out));
    token0.trade_volume_usd =
        token0.trade_volume_usd.add(tracked_amount_usd);
    token0.untracked_volume_usd =
        token0.untracked_volume_usd.add(derived_amount_usd);

    token1.trade_volume =
        token1.trade_volume.add(amount1_in.add(amount1_out));
    token1.trade_volume_usd =
        token1.trade_volume_usd.add(tracked_amount_usd);
    token1.untracked_volume_usd =
        token1.untracked_volume_usd.add(derived_amount_usd);

    token0.tx_count += 1;
    token1.tx_count += 1;

    pair.volume_usd = pair.volume_usd.add(tracked_amount_usd);
    pair.volume_token0 = pair.volume_token0.add(amount0_total);
    pair.volume_token1 = pair.volume_token1.add(amount1_total);
    pair.untracked_volume_usd =
        pair.untracked_volume_usd.add(derived_amount_usd);
    pair.tx_count += 1;

    db.update_pair(&pair).await;

    let mut factory = db.get_factory().await;
    factory.total_volume_usd =
        factory.total_volume_usd.add(tracked_amount_usd);
    factory.total_liquidity_eth =
        factory.total_volume_eth.add(tracked_amount_eth);
    factory.tx_count += 1;

    db.update_pair(&pair).await;

    db.update_token(&token0).await;
    db.update_token(&token1).await;
    db.update_factory(&factory).await;

    let transaction_hash = log.transaction_hash.unwrap().to_string();
    let block_number = log.block_number.unwrap() as i64;
    let block_timestamp = log.block_timestamp.unwrap() as i64;

    let mut transaction = match db.get_transaction(&transaction_hash).await
    {
        Some(transaction) => transaction,
        None => DatabaseTransaction::new(
            log.transaction_hash.unwrap().to_string(),
            block_number,
            block_timestamp,
        ),
    };

    let swap_id = format!(
        "{}-{}",
        transaction_hash.clone(),
        transaction.swaps.len()
    );

    let amount_usd = match tracked_amount_usd == udec256!(0) {
        true => derived_amount_usd,
        false => tracked_amount_usd,
    };

    let swap = DatabaseSwap::new(
        swap_id,
        SwapData {
            pair: pair.id.clone(),
            sender: event.sender.to_string(),
            to: event.to.to_string(),
            from: "".to_string(), // TODO: get 'from'
            log_index: log.log_index.unwrap() as i64,
            transaction: log.transaction_hash.unwrap().to_string(),
            timestamp: block_timestamp,
        },
        SwapAmounts {
            amount0_in,
            amount1_in,
            amount0_out,
            amount1_out,
            amount_usd,
        },
    );

    db.update_swap(&swap).await;

    transaction.swaps.push(swap.id.clone());

    db.update_transaction(&transaction).await;

    let mut pair_day_data = update_pair_day_data(&log, db).await;
    let mut pair_hour_data = update_pair_hour_data(&log, db).await;
    let mut factory_day_data = update_factory_day_data(&log, db).await;
    let mut token0_day_data =
        update_token_day_data(&token0, &log, db).await;
    let mut token1_day_data =
        update_token_day_data(&token1, &log, db).await;

    factory_day_data.daily_volume_usd =
        factory_day_data.daily_volume_usd.add(tracked_amount_usd);
    factory_day_data.daily_volume_eth =
        factory_day_data.daily_volume_eth.add(tracked_amount_eth);
    factory_day_data.daily_volume_untracked =
        factory_day_data.daily_volume_untracked.add(derived_amount_usd);

    db.update_factory_day_data(&factory_day_data).await;

    pair_day_data.daily_volume_token0 =
        pair_day_data.daily_volume_token0.add(amount0_total);
    pair_day_data.daily_volume_token1 =
        pair_day_data.daily_volume_token1.add(amount1_total);
    pair_day_data.daily_volume_usd =
        pair_day_data.daily_volume_usd.add(tracked_amount_usd);

    db.update_pair_day_data(&pair_day_data).await;

    pair_hour_data.hourly_volume_token0 =
        pair_hour_data.hourly_volume_token0.add(amount0_total);
    pair_hour_data.hourly_volume_token1 =
        pair_hour_data.hourly_volume_token1.add(amount1_total);
    pair_hour_data.hourly_volume_usd =
        pair_hour_data.hourly_volume_usd.add(tracked_amount_usd);

    db.update_pair_hour_data(&pair_hour_data).await;

    token0_day_data.daily_volume_token =
        token0_day_data.daily_volume_token.add(amount0_total);
    token0_day_data.daily_volume_eth = token0_day_data
        .daily_volume_eth
        .add(amount0_total.mul(token0.derived_eth));
    token0_day_data.daily_volume_usd = token0_day_data
        .daily_volume_usd
        .add(amount0_total.mul(token0.derived_eth).mul(bundle.eth_price));

    db.update_token_day_data(&token0_day_data).await;

    token1_day_data.daily_volume_token =
        token1_day_data.daily_volume_token.add(amount1_total);
    token1_day_data.daily_volume_eth = token1_day_data
        .daily_volume_eth
        .add(amount1_total.mul(token1.derived_eth));
    token1_day_data.daily_volume_usd = token1_day_data
        .daily_volume_usd
        .add(amount1_total.mul(token1.derived_eth).mul(bundle.eth_price));

    db.update_token_day_data(&token1_day_data).await;
}
