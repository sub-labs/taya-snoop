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

    for log in pairs {
        let event = PairCreated::decode_log(&log.inner, true).unwrap();

        let token0_address = event.token0.to_string().to_lowercase();
        let token1_address = event.token1.to_string().to_lowercase();
        let pair_address = event.pair.to_string().to_lowercase();

        let (token0, token1) = tokio::join!(
            db.get_token(&token0_address),
            db.get_token(&token1_address)
        );

        let mut factory = db.get_factory().await;

        factory.pair_count += 1;
        factory.pairs.push(Some(pair_address));

        if token0.is_none() {
            let (name, symbol, total_supply, decimals) =
                rpc.get_token_information(token0_address.clone()).await;

            let token = DatabaseToken::new(
                token0_address,
                symbol,
                name,
                decimals,
                total_supply,
            );

            db.update_token(&token).await;
            count_tokens += 1;
        }

        if token1.is_none() {
            let (name, symbol, total_supply, decimals) =
                rpc.get_token_information(token1_address.clone()).await;

            let token = DatabaseToken::new(
                token1_address,
                symbol,
                name,
                decimals,
                total_supply,
            );

            db.update_token(&token).await;

            count_tokens += 1;
        }

        let block_number = log.block_number.unwrap() as i32;
        let block_timestamp = log.block_timestamp.unwrap() as i32;

        let pair = DatabasePair::new(event, block_timestamp, block_number);

        tokio::join!(db.update_factory(&factory), db.update_pair(&pair));
    }

    info!("Stored {} pairs and {} tokens", count_pairs, count_tokens);
}
