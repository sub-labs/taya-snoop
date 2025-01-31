use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use log::info;

use crate::{
    db::{
        models::{pair::DatabasePair, token::DatabaseToken},
        Database,
    },
    rpc::Rpc,
};

sol! {
    event PairCreated(address indexed token0, address indexed token1, address pair, uint);
}

pub async fn handle_pairs(pairs: Vec<Log>, db: &Database, rpc: &Rpc) {
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
        // Load the token1
        let token1 = db.get_token(event.token1.to_string()).await;

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
