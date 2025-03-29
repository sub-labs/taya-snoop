// @generated automatically by Diesel CLI.

diesel::table! {
    bundles (id) {
        id -> Text,
        eth_price -> Numeric,
    }
}

diesel::table! {
    burns (id) {
        id -> Text,
        transaction -> Text,
        timestamp -> Int4,
        pair -> Text,
        liquidity -> Numeric,
        sender -> Text,
        amount0 -> Numeric,
        amount1 -> Numeric,
        to -> Text,
        log_index -> Int4,
        amount_usd -> Numeric,
        needs_complete -> Bool,
        fee_to -> Text,
        fee_liquidity -> Numeric,
    }
}

diesel::table! {
    dex_day_data (id) {
        id -> Text,
        date -> Int4,
        daily_volume_eth -> Numeric,
        daily_volume_usd -> Numeric,
        daily_volume_untracked -> Numeric,
        total_volume_eth -> Numeric,
        total_liquidity_eth -> Numeric,
        total_volume_usd -> Numeric,
        total_liquidity_usd -> Numeric,
        tx_count -> Int4,
    }
}

diesel::table! {
    factories (id) {
        id -> Text,
        pair_count -> Int4,
        pairs -> Array<Nullable<Text>>,
        total_volume_usd -> Numeric,
        total_volume_eth -> Numeric,
        untracked_volume_usd -> Numeric,
        total_liquidity_usd -> Numeric,
        total_liquidity_eth -> Numeric,
        tx_count -> Int4,
    }
}

diesel::table! {
    mints (id) {
        id -> Text,
        transaction -> Text,
        timestamp -> Int4,
        pair -> Text,
        to -> Text,
        liquidity -> Numeric,
        sender -> Text,
        amount0 -> Numeric,
        amount1 -> Numeric,
        log_index -> Int4,
        amount_usd -> Numeric,
        fee_to -> Text,
        fee_liquidity -> Numeric,
    }
}

diesel::table! {
    pair_day_data (id) {
        id -> Text,
        date -> Int4,
        pair_address -> Text,
        token0 -> Text,
        token1 -> Text,
        reserve0 -> Numeric,
        reserve1 -> Numeric,
        total_supply -> Numeric,
        reserve_usd -> Numeric,
        daily_volume_token0 -> Numeric,
        daily_volume_token1 -> Numeric,
        daily_volume_usd -> Numeric,
        daily_txns -> Int4,
    }
}

diesel::table! {
    pair_hour_data (id) {
        id -> Text,
        hour_start_unix -> Int4,
        pair -> Text,
        reserve0 -> Numeric,
        reserve1 -> Numeric,
        total_supply -> Numeric,
        reserve_usd -> Numeric,
        hourly_volume_token0 -> Numeric,
        hourly_volume_token1 -> Numeric,
        hourly_volume_usd -> Numeric,
        hourly_txns -> Int4,
    }
}

diesel::table! {
    pairs (id) {
        id -> Text,
        token0 -> Text,
        token1 -> Text,
        reserve0 -> Numeric,
        reserve1 -> Numeric,
        total_supply -> Numeric,
        reserve_eth -> Numeric,
        reserve_usd -> Numeric,
        tracked_reserve_eth -> Numeric,
        token0_price -> Numeric,
        token1_price -> Numeric,
        volume_token0 -> Numeric,
        volume_token1 -> Numeric,
        volume_usd -> Numeric,
        untracked_volume_usd -> Numeric,
        tx_count -> Int4,
        created_at_timestamp -> Int4,
        created_at_block_number -> Int4,
        liquidity_provider_count -> Int4,
    }
}

diesel::table! {
    swaps (id) {
        id -> Text,
        transaction -> Text,
        timestamp -> Int4,
        pair -> Text,
        sender -> Text,
        from -> Text,
        amount0_in -> Numeric,
        amount1_in -> Numeric,
        amount0_out -> Numeric,
        amount1_out -> Numeric,
        to -> Text,
        log_index -> Int4,
        amount_usd -> Numeric,
    }
}

diesel::table! {
    sync_state (id) {
        id -> Text,
        last_block_indexed -> Int4,
    }
}

diesel::table! {
    token_day_data (id) {
        id -> Text,
        date -> Int4,
        token -> Text,
        daily_volume_token -> Numeric,
        daily_volume_eth -> Numeric,
        daily_volume_usd -> Numeric,
        daily_txns -> Int4,
        total_liquidity_token -> Numeric,
        total_liquidity_eth -> Numeric,
        total_liquidity_usd -> Numeric,
        price_usd -> Numeric,
    }
}

diesel::table! {
    tokens (id) {
        id -> Text,
        symbol -> Text,
        name -> Text,
        decimals -> Int4,
        total_supply -> Numeric,
        trade_volume -> Numeric,
        trade_volume_usd -> Numeric,
        untracked_volume_usd -> Numeric,
        tx_count -> Int4,
        total_liquidity -> Numeric,
        derived_eth -> Numeric,
    }
}

diesel::table! {
    transactions (id) {
        id -> Text,
        block_number -> Int4,
        timestamp -> Int4,
        mints -> Array<Nullable<Text>>,
        swaps -> Array<Nullable<Text>>,
        burns -> Array<Nullable<Text>>,
    }
}

diesel::joinable!(pair_day_data -> pairs (pair_address));
diesel::joinable!(pair_hour_data -> pairs (pair));
diesel::joinable!(token_day_data -> tokens (token));

diesel::allow_tables_to_appear_in_same_query!(
    bundles,
    burns,
    dex_day_data,
    factories,
    mints,
    pair_day_data,
    pair_hour_data,
    pairs,
    swaps,
    sync_state,
    token_day_data,
    tokens,
    transactions,
);
