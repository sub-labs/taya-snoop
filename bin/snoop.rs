use alloy::primitives::map::HashMap;
use eth_snoop::{
    configs::Config,
    db::{models::token::DatabaseToken, Database, StoreData},
    rpc::Rpc,
};
use log::*;
use simple_logger::SimpleLogger;

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
    let mut tokens: HashMap<String, DatabaseToken> = HashMap::default();

    let mut last_synced_block = db.get_last_block_indexed();

    if last_synced_block < config.factory.start_block {
        last_synced_block = config.factory.start_block
    }

    let last_chain_block = rpc.get_last_block().await;

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

        let (logs, events) = rpc
            .get_factory_logs_batch(first_block, last_block, config)
            .await;

        info!(
            "Getting logs between blocks {} and {}",
            first_block, last_block
        );

        let mut db_tokens: Vec<DatabaseToken> = vec![];

        if !logs.is_empty() {
            // Fetch the data of the pair tokens
            for pair in events.clone().into_iter() {
                if !tokens.contains_key(&pair.token0) {
                    let token_data =
                        rpc.get_token_information(pair.token0).await;

                    db_tokens.push(token_data.clone());

                    tokens.insert(token_data.address.clone(), token_data);
                }

                if !tokens.contains_key(&pair.token1) {
                    let token_data =
                        rpc.get_token_information(pair.token1).await;

                    db_tokens.push(token_data.clone());

                    tokens.insert(token_data.address.clone(), token_data);
                }
            }

            let store_data =
                StoreData { logs, pairs: events, tokens: db_tokens };

            db.store_data(store_data).await;
        }

        db.update_last_block_indexed(last_block);
    }
}
