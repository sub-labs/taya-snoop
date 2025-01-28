use eth_snoop::{
    configs::Config,
    db::{Database, StoreData},
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
    let mut last_synced_block = db.get_last_block_indexed();

    if last_synced_block < config.contract.start_block {
        last_synced_block = config.contract.start_block
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

        let logs =
            rpc.get_logs_batch(first_block, last_block, config).await;

        info!(
            "Getting logs between blocks {} and {}",
            first_block, last_block
        );

        if !logs.is_empty() {
            // [Custom setup]: Here is the perfect place to run your logic and generate new models based on your events.

            let store_data = StoreData { logs };

            db.store_data(store_data).await;
        }

        db.update_last_block_indexed(last_block);
    }
}
