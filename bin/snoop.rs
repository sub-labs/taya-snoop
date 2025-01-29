use log::*;
use simple_logger::SimpleLogger;
use taya_snoop::{
    configs::Config,
    db::{
        models::{
            burn::DatabaseBurn, log::DatabaseLog, mint::DatabaseMint,
            pair::DatabasePair, swap::DatabaseSwap,
        },
        Database,
    },
    rpc::Rpc,
};
use tokio::join;

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

    let mut factory = db.get_factory(config).await;

    for block_chunk in sync_blocks_chunks {
        let first_block = block_chunk[0];
        let last_block = block_chunk[block_chunk.len() - 1];

        let (mut factory_logs, pairs) = rpc
            .get_factory_logs_batch(
                first_block as u64,
                last_block as u64,
                config,
            )
            .await;

        let new_pair_ids: Vec<String> =
            pairs.iter().map(|pair| pair.id.to_string()).collect();

        factory.pairs.append(&mut new_pair_ids.clone());
        factory.pair_count += new_pair_ids.len() as i32;

        let (mut pair_logs, mints, burns, swaps, syncs) = rpc
            .get_pairs_logs_batch(
                &factory.pairs,
                first_block as u64,
                last_block as u64,
                &config.chain,
            )
            .await;

        info!(
            "Getting logs between blocks {} and {}",
            first_block, last_block
        );

        factory_logs.append(&mut pair_logs);

        join!(
            db.store::<DatabaseLog>(
                taya_snoop::db::DatabaseKeys::Logs,
                &factory_logs,
            ),
            db.store::<DatabaseMint>(
                taya_snoop::db::DatabaseKeys::Mints,
                &mints,
            ),
            db.store::<DatabaseBurn>(
                taya_snoop::db::DatabaseKeys::Burns,
                &burns,
            ),
            db.store::<DatabaseSwap>(
                taya_snoop::db::DatabaseKeys::Swaps,
                &swaps,
            ),
            db.store::<DatabasePair>(
                taya_snoop::db::DatabaseKeys::Pairs,
                &pairs,
            )
        );

        info!("Stored logs ({}) mints ({}), burns ({}) swaps ({}) pairs ({})", factory_logs.len(), mints.len(), burns.len(), swaps.len(), pairs.len());

        db.update_factory(&factory).await;
        db.update_last_block_indexed(last_block).await;
    }
}
