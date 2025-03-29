CREATE TABLE factories (
    id TEXT PRIMARY KEY,
    pair_count INTEGER NOT NULL,
    total_volume_usd NUMERIC NOT NULL,
    total_volume_eth NUMERIC NOT NULL,
    untracked_volume_usd NUMERIC NOT NULL,
    total_liquidity_usd NUMERIC NOT NULL,
    total_liquidity_eth NUMERIC NOT NULL,
    tx_count INTEGER NOT NULL,
    pairs TEXT[] NOT NULL
);

CREATE TABLE tokens (
    id TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    name TEXT NOT NULL,
    decimals INTEGER NOT NULL,
    total_supply NUMERIC NOT NULL,
    trade_volume NUMERIC NOT NULL,
    trade_volume_usd NUMERIC NOT NULL,
    untracked_volume_usd NUMERIC NOT NULL,
    tx_count INTEGER NOT NULL,
    total_liquidity NUMERIC NOT NULL,
    derived_eth NUMERIC NOT NULL
);

CREATE TABLE pairs (
    id TEXT PRIMARY KEY,
    token0 TEXT NOT NULL,
    token1 TEXT NOT NULL,
    reserve0 NUMERIC NOT NULL,
    reserve1 NUMERIC NOT NULL,
    total_supply NUMERIC NOT NULL,
    reserve_eth NUMERIC NOT NULL,
    reserve_usd NUMERIC NOT NULL,
    tracked_reserve_eth NUMERIC NOT NULL,
    token0_price NUMERIC NOT NULL,
    token1_price NUMERIC NOT NULL,
    volume_token0 NUMERIC NOT NULL,
    volume_token1 NUMERIC NOT NULL,
    volume_usd NUMERIC NOT NULL,
    untracked_volume_usd NUMERIC NOT NULL,
    tx_count INTEGER NOT NULL,
    created_at_timestamp INTEGER NOT NULL,
    created_at_block_number INTEGER NOT NULL,
    liquidity_provider_count INTEGER NOT NULL,
    CONSTRAINT token0_data FOREIGN KEY (token0) REFERENCES tokens(id),
    CONSTRAINT token1_data FOREIGN KEY (token1) REFERENCES tokens(id)
);

CREATE TABLE transactions (
    id TEXT PRIMARY KEY,
    block_number INTEGER NOT NULL,
    timestamp INTEGER NOT NULL,
    mints TEXT[] NOT NULL,
    swaps TEXT[] NOT NULL,
    burns TEXT[] NOT NULL
);

CREATE TABLE mints (
    id TEXT PRIMARY KEY,
    transaction TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    pair TEXT NOT NULL,
    "to" TEXT NOT NULL,
    liquidity NUMERIC NOT NULL,
    sender TEXT NOT NULL,
    amount0 NUMERIC NOT NULL,
    amount1 NUMERIC NOT NULL,
    log_index INTEGER NOT NULL,
    amount_usd NUMERIC NOT NULL,
    fee_to TEXT NOT NULL,
    fee_liquidity NUMERIC NOT NULL,
    CONSTRAINT transaction_data FOREIGN KEY (transaction) REFERENCES transactions(id),
    CONSTRAINT pair_data FOREIGN KEY (pair) REFERENCES pairs(id)
);

CREATE TABLE burns (
    id TEXT PRIMARY KEY,
    transaction TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    pair TEXT NOT NULL,
    liquidity NUMERIC NOT NULL,
    sender TEXT NOT NULL,
    amount0 NUMERIC NOT NULL,
    amount1 NUMERIC NOT NULL,
    "to" TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    amount_usd NUMERIC NOT NULL,
    needs_complete BOOLEAN NOT NULL,
    fee_to TEXT NOT NULL,
    fee_liquidity NUMERIC NOT NULL,
    CONSTRAINT transaction_data FOREIGN KEY (transaction) REFERENCES transactions(id),
    CONSTRAINT pair_data FOREIGN KEY (pair) REFERENCES pairs(id)
);

CREATE TABLE swaps (
    id TEXT PRIMARY KEY,
    transaction TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    pair TEXT NOT NULL,
    sender TEXT NOT NULL,
    "from" TEXT NOT NULL,
    amount0_in NUMERIC NOT NULL,
    amount1_in NUMERIC NOT NULL,
    amount0_out NUMERIC NOT NULL,
    amount1_out NUMERIC NOT NULL,
    "to" TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    amount_usd NUMERIC NOT NULL,
    CONSTRAINT transaction_data FOREIGN KEY (transaction) REFERENCES transactions(id),
    CONSTRAINT pair_data FOREIGN KEY (pair) REFERENCES pairs(id)
);

CREATE TABLE bundles (
    id TEXT PRIMARY KEY,
    eth_price NUMERIC NOT NULL
);

CREATE TABLE dex_day_data (
    id TEXT PRIMARY KEY,
    date INTEGER NOT NULL,
    daily_volume_eth NUMERIC NOT NULL,
    daily_volume_usd NUMERIC NOT NULL,
    daily_volume_untracked NUMERIC NOT NULL,
    total_volume_eth NUMERIC NOT NULL,
    total_liquidity_eth NUMERIC NOT NULL,
    total_volume_usd NUMERIC NOT NULL,
    total_liquidity_usd NUMERIC NOT NULL,
    tx_count INTEGER NOT NULL
);

CREATE TABLE pair_hour_data (
    id TEXT PRIMARY KEY,
    hour_start_unix INTEGER NOT NULL,
    pair TEXT NOT NULL,
    reserve0 NUMERIC NOT NULL,
    reserve1 NUMERIC NOT NULL,
    total_supply NUMERIC NOT NULL,
    reserve_usd NUMERIC NOT NULL,
    hourly_volume_token0 NUMERIC NOT NULL,
    hourly_volume_token1 NUMERIC NOT NULL,
    hourly_volume_usd NUMERIC NOT NULL,
    hourly_txns INTEGER NOT NULL,
    CONSTRAINT pair_data FOREIGN KEY (pair) REFERENCES pairs(id)
);


CREATE TABLE pair_day_data (
    id TEXT PRIMARY KEY,
    date INTEGER NOT NULL,
    pair_address TEXT NOT NULL,
    token0 TEXT NOT NULL,
    token1 TEXT NOT NULL,
    reserve0 NUMERIC NOT NULL,
    reserve1 NUMERIC NOT NULL,
    total_supply NUMERIC NOT NULL,
    reserve_usd NUMERIC NOT NULL,
    daily_volume_token0 NUMERIC NOT NULL,
    daily_volume_token1 NUMERIC NOT NULL,
    daily_volume_usd NUMERIC NOT NULL,
    daily_txns INTEGER NOT NULL,
    CONSTRAINT token0_data FOREIGN KEY (token0) REFERENCES tokens(id),
    CONSTRAINT token1_data FOREIGN KEY (token1) REFERENCES tokens(id),
    CONSTRAINT pair_data FOREIGN KEY (pair_address) REFERENCES pairs(id)
);

CREATE TABLE token_day_data (
    id TEXT PRIMARY KEY,
    date INTEGER NOT NULL,
    token TEXT NOT NULL,
    daily_volume_token NUMERIC NOT NULL,
    daily_volume_eth NUMERIC NOT NULL,
    daily_volume_usd NUMERIC NOT NULL,
    daily_txns INTEGER NOT NULL,
    total_liquidity_token NUMERIC NOT NULL,
    total_liquidity_eth NUMERIC NOT NULL,
    total_liquidity_usd NUMERIC NOT NULL,
    price_usd NUMERIC NOT NULL,
    CONSTRAINT token_data FOREIGN KEY (token) REFERENCES tokens(id)
);

CREATE TABLE sync_state (
    id TEXT PRIMARY KEY,
    last_block_indexed INTEGER NOT NULL
);

