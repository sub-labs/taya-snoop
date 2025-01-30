use alloy::{
    primitives::{utils::format_units, Address, U256},
    rpc::types::Log,
    sol_types::SolEvent,
};
use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use taya_snoop::{
    configs::Config,
    db::{
        models::{
            factory::PairCreated,
            pair::{DatabasePair, Sync},
            token::DatabaseToken,
        },
        Database,
    },
    rpc::Rpc,
};

#[tokio::main()]
async fn main() {
    let log = SimpleLogger::new().with_level(LevelFilter::Info);

    let config = Config::new();

    if config.debug {
        log.with_level(LevelFilter::Debug).init().unwrap();
    } else {
        log.init().unwrap();
    }

    let rpc = Rpc::new(&config).await;

    let db =
        Database::new(config.db_url.clone(), config.chain.clone()).await;

    info!("Starting Taya Snoop.");

    loop {
        sync_chain(&rpc, &db, &config).await;
    }
}

async fn sync_chain(rpc: &Rpc, db: &Database, config: &Config) {
    let mut last_synced_block = db.get_last_block_indexed().await;

    if last_synced_block < config.factory.start_block {
        last_synced_block = config.factory.start_block
    }

    let last_chain_block = rpc.get_last_block().await as i64;

    let sync_blocks: Vec<i64> =
        (last_synced_block + 1..=last_chain_block).collect();

    let sync_blocks_chunks: std::slice::Chunks<'_, i64> =
        sync_blocks.chunks(config.batch_size);

    info!(
        "Start sync from block {} to {} with {} blocks each batch",
        last_synced_block, last_chain_block, config.batch_size
    );

    for block_chunk in sync_blocks_chunks {
        let first_block = block_chunk[0];
        let last_block = block_chunk[block_chunk.len() - 1];

        let pair_logs = rpc
            .get_factory_logs_batch(
                first_block as u64,
                last_block as u64,
                config,
            )
            .await;

        handle_pairs(pair_logs, db, rpc).await;

        let pairs = db.get_pairs().await;

        if !pairs.is_empty() {
            let (mints, burns, swaps, syncs, transfers) = rpc
                .get_pairs_logs_batch(
                    &pairs,
                    first_block as u64,
                    last_block as u64,
                )
                .await;

            handle_mints(&mints, db).await;

            handle_burns(&burns, db).await;

            handle_swaps(&swaps, db).await;

            handle_syncs(&syncs, db, rpc, config).await;

            handle_transfers(&transfers, db).await;

            info!("Procesed {} mints {} burns {} swaps {} sync and {} transfer events", mints.len(), burns.len(), swaps.len(), syncs.len(), transfers.len());
        }

        db.update_last_block_indexed(last_block).await;
    }
}

async fn handle_pairs(pairs: Vec<Log>, db: &Database, rpc: &Rpc) {
    let mut count_tokens = 0;
    let count_pairs = pairs.len();

    // Start iterating pair events
    for log in pairs {
        // Parse the log to event
        let event = PairCreated::decode_log(&log.inner, true).unwrap();

        // Load the factory
        let mut factory = db.get_factory().await;

        // Add the pair to the count
        factory.pair_count += 1;
        factory.pairs.push(event.pair.to_string());

        // Load the token0
        let token0 = db.get_token(event.token0.to_string()).await;
        // Create if it doesn't exists
        if token0.is_none() {
            let (name, symbol, total_supply, decimals) =
                rpc.get_token_information(event.token0.to_string()).await;

            let token = DatabaseToken::new(
                event.token0.to_string(),
                symbol,
                name,
                decimals,
                total_supply,
            );

            db.update_token(&token).await;
            count_tokens += 1;
        }

        // Load the token1
        let token1 = db.get_token(event.token1.to_string()).await;
        // Create if it doesn't exists
        if token1.is_none() {
            let (name, symbol, total_supply, decimals) =
                rpc.get_token_information(event.token1.to_string()).await;

            let token = DatabaseToken::new(
                event.token1.to_string(),
                symbol,
                name,
                decimals,
                total_supply,
            );

            db.update_token(&token).await;
            count_tokens += 1;
        }

        // Create the pair data
        let db_pair = DatabasePair::new(
            event,
            log.block_timestamp.unwrap_or(0) as i64,
            log.block_number.unwrap_or(0) as i64,
        );

        // Store the factory and the new pair
        db.update_factory(&factory).await;
        db.update_pair(&db_pair).await;
    }

    info!("Stored {} pairs and {} tokens", count_pairs, count_tokens);
}

async fn handle_mints(mints: &Vec<Log>, db: &Database) {}

async fn handle_burns(burns: &Vec<Log>, db: &Database) {}

async fn handle_swaps(swap: &Vec<Log>, db: &Database) {}

async fn handle_syncs(
    syncs: &Vec<Log>,
    db: &Database,
    rpc: &Rpc,
    config: &Config,
) {
    for log in syncs {
        let event = Sync::decode_log(&log.inner, true).unwrap();

        // Get the pair
        let mut pair =
            db.get_pair(event.address.to_string()).await.unwrap();

        // Get the token0
        let mut token0 = db.get_token(pair.token0.clone()).await.unwrap();
        // Get the token1
        let mut token1 = db.get_token(pair.token1.clone()).await.unwrap();

        // Load the factory
        let mut factory = db.get_factory().await;

        factory.total_liquidity_eth -= pair.tracked_reserve_eth.clone();

        token0.total_liquidity -= pair.reserve0;
        token1.total_liquidity -= pair.reserve1;

        pair.reserve0 = BigDecimal::from_f64(
            format_units(
                U256::from(event.reserve0),
                token0.decimals as u8,
            )
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        )
        .unwrap();

        pair.reserve1 = BigDecimal::from_f64(
            format_units(
                U256::from(event.reserve1),
                token0.decimals as u8,
            )
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        )
        .unwrap();

        if pair.reserve0.ne(&BigDecimal::zero()) {
            pair.token0_price =
                pair.reserve0.clone() / pair.reserve1.clone()
        } else {
            pair.token0_price = BigDecimal::zero()
        }

        if pair.reserve1.ne(&BigDecimal::zero()) {
            pair.token1_price =
                pair.reserve1.clone() / pair.reserve0.clone()
        } else {
            pair.token1_price = BigDecimal::zero()
        }

        db.update_pair(&pair).await;

        let mut bundle = db.get_bundle().await;

        bundle.eth_price = get_eth_price_usd(db).await;

        db.update_bundle(&bundle).await;

        token0.derived_eth =
            find_eth_per_token(&token0, rpc, db, config).await;

        token1.derived_eth =
            find_eth_per_token(&token1, rpc, db, config).await;

        db.update_token(&token0).await;
        db.update_token(&token1).await;

        let mut tracked_liquidity_eth = BigDecimal::zero();
        if bundle.eth_price.ne(&BigDecimal::zero()) {
            tracked_liquidity_eth = get_tracked_liquidity_usd(
                pair.reserve0.clone(),
                &token0,
                pair.reserve1.clone(),
                &token1,
                db,
            )
            .await
        }

        pair.tracked_reserve_eth = tracked_liquidity_eth.clone();
        pair.reserve_eth = (pair.reserve0.clone()
            * token0.derived_eth.clone())
            + (pair.reserve1.clone() * token1.derived_eth.clone());

        pair.reserve_usd =
            pair.reserve_eth.clone() * bundle.eth_price.clone();

        factory.total_liquidity_eth += tracked_liquidity_eth;
        factory.total_liquidity_usd =
            factory.total_liquidity_eth.clone() * bundle.eth_price;

        token0.total_liquidity += pair.reserve0.clone();
        token1.total_liquidity += pair.reserve1.clone();

        db.update_pair(&pair).await;
        db.update_factory(&factory).await;
        db.update_token(&token0).await;
        db.update_token(&token1).await;
    }
}

async fn handle_transfers(transfers: &Vec<Log>, db: &Database) {}

const MINIMUM_LIQUIDITY_THRESHOLD_ETH: f64 = 2.0;

const WHITELIST_TOKENS: [&str; 4] = [
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
    "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
    "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
];

pub const WETH_ADDRESS: &str =
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub const DAI_WETH_PAIR: &str =
    "0xa478c2975ab1ea89e8196811f51a7b7ade33eb11";
pub const USDC_WETH_PAIR: &str =
    "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc";
pub const USDT_WETH_PAIR: &str =
    "0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852";

async fn get_eth_price_usd(db: &Database) -> BigDecimal {
    let dai_pair = db.get_pair(DAI_WETH_PAIR.to_owned()).await;
    let usdc_pair = db.get_pair(USDC_WETH_PAIR.to_owned()).await;
    let usdt_pair = db.get_pair(USDT_WETH_PAIR.to_owned()).await;

    match (dai_pair, usdc_pair, usdt_pair) {
        (Some(dai), Some(usdc), Some(usdt)) => {
            let total_liquidity_eth = dai.reserve1.clone()
                + usdc.reserve1.clone()
                + usdt.reserve0.clone();

            if total_liquidity_eth.eq(&BigDecimal::zero()) {
                return BigDecimal::zero();
            }

            let dai_weight =
                dai.reserve1.clone() / total_liquidity_eth.clone();
            let usdc_weight =
                usdc.reserve1.clone() / total_liquidity_eth.clone();
            let usdt_weight =
                usdt.reserve0.clone() / total_liquidity_eth.clone();

            (dai.token0_price * dai_weight)
                + (usdc.token0_price * usdc_weight)
                + (usdt.token1_price * usdt_weight)
        }
        (Some(dai), Some(usdc), None) => {
            let total_liquidity_eth =
                dai.reserve1.clone() + usdc.reserve1.clone();

            if total_liquidity_eth.eq(&BigDecimal::zero()) {
                return BigDecimal::zero();
            }

            let dai_weight =
                dai.reserve1.clone() / total_liquidity_eth.clone();
            let usdc_weight =
                usdc.reserve1.clone() / total_liquidity_eth.clone();

            (dai.token0_price * dai_weight)
                + (usdc.token0_price * usdc_weight)
        }
        (_, Some(usdc), _) => usdc.token0_price,
        // No pairs
        _ => BigDecimal::zero(),
    }
}

async fn find_eth_per_token(
    token: &DatabaseToken,
    rpc: &Rpc,
    db: &Database,
    config: &Config,
) -> BigDecimal {
    if token.id == WETH_ADDRESS {
        return BigDecimal::one();
    }

    // Loop through a set of whitelisted tokens to check if there is any pair for this token.
    for whitelist_token in WHITELIST_TOKENS {
        let pair_address = rpc
            .get_pair_for_tokens(
                token.id.clone(),
                whitelist_token.to_owned(),
                config,
            )
            .await;

        if pair_address != Address::ZERO.to_string() {
            let pair = db.get_pair(pair_address).await;
            if pair.is_none() {
                continue;
            }

            let pair = pair.unwrap();
            let minimum_liquidity =
                BigDecimal::from_f64(MINIMUM_LIQUIDITY_THRESHOLD_ETH)
                    .unwrap();

            if pair.token0 == token.id
                && pair.reserve_eth.ge(&minimum_liquidity)
            {
                let token0 = db.get_token(pair.token0).await;
                if token0.is_none() {
                    continue;
                }

                let token0 = token0.unwrap();

                return pair.token0_price * token0.derived_eth;
            }
            if pair.token1 == token.id
                && pair.reserve_eth.ge(&minimum_liquidity)
            {
                let token1 = db.get_token(pair.token1).await;
                if token1.is_none() {
                    continue;
                }

                let token1 = token1.unwrap();

                return pair.token1_price * token1.derived_eth;
            }
        }
    }

    BigDecimal::zero()
}

async fn get_tracked_liquidity_usd(
    token_amount0: BigDecimal,
    token0: &DatabaseToken,
    token_amount1: BigDecimal,
    token1: &DatabaseToken,
    db: &Database,
) -> BigDecimal {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.clone() * bundle.eth_price.clone();
    let price1 = token1.derived_eth.clone() * bundle.eth_price.clone();

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount0 * price0) + (token_amount1 * price1);
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount0 * price0)
            * BigDecimal::from_usize(2).unwrap();
    }

    if !WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount1 * price1)
            * BigDecimal::from_usize(2).unwrap();
    }

    BigDecimal::zero()
}
