use std::{thread, time};

use alloy::sol_types::SolEvent;
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use taya_snoop::{
    configs::Config,
    db::Database,
    handlers::{
        burn::{handle_burn, Burn},
        mint::{handle_mint, Mint},
        pairs::handle_pairs,
        swap::{handle_swap, Swap},
        sync::{handle_sync, Sync},
        transfer::{handle_transfer, Transfer},
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
        thread::sleep(time::Duration::from_secs(10));
    }
}

async fn sync_chain(rpc: &Rpc, db: &Database, config: &Config) {
    let mut last_synced_block = db.get_last_block_indexed().await;

    if last_synced_block < config.chain.start_block {
        last_synced_block = config.chain.start_block
    }

    let last_chain_block = match rpc.get_last_block().await {
        Some(last_chain_block) => last_chain_block,
        None => return,
    };

    let sync_blocks: Vec<i32> =
        (last_synced_block + 1..=last_chain_block).collect();

    let sync_blocks_chunks: std::slice::Chunks<'_, i32> =
        sync_blocks.chunks(config.batch_size);

    info!(
        "Start sync from block {} to {} with {} blocks each batch",
        last_synced_block, last_chain_block, config.batch_size
    );

    for block_chunk in sync_blocks_chunks {
        let first_block = block_chunk[0];
        let last_block = block_chunk[block_chunk.len() - 1];

        let pair_logs = match rpc
            .get_factory_logs_batch(
                first_block as u64,
                last_block as u64,
                config,
            )
            .await
        {
            Some(logs) => logs,
            None => return,
        };

        let (mut factory, mut bundle) =
            tokio::join!(db.get_factory(), db.get_bundle());

        handle_pairs(&mut factory, pair_logs, db, rpc).await;

        if !factory.pairs.is_empty() {
            let pairs: Vec<String> = factory
                .pairs
                .clone()
                .into_iter()
                .map(|pair| pair.unwrap())
                .collect();

            let mut event_logs = match rpc
                .get_pairs_logs_batch(&pairs, first_block, last_block)
                .await
            {
                Some(logs) => logs,
                None => return,
            };

            let mut count_mints = 0;
            let mut count_burns = 0;
            let mut count_swaps = 0;
            let mut count_syncs = 0;
            let mut count_transfers = 0;

            event_logs.sort_unstable_by_key(|log| {
                let block_number = log.block_number.unwrap();
                let log_index = log.log_index.unwrap();
                (block_number, log_index)
            });

            for log in event_logs {
                match log.topic0() {
                    Some(topic_raw) => {
                        let block_timestamp =
                            log.block_timestamp.unwrap() as i32;

                        let topic = topic_raw.to_string();

                        if topic == Mint::SIGNATURE_HASH.to_string() {
                            handle_mint(
                                log,
                                block_timestamp,
                                db,
                                &mut factory,
                                &bundle,
                            )
                            .await;
                            count_mints += 1;
                        } else if topic == Burn::SIGNATURE_HASH.to_string()
                        {
                            handle_burn(
                                log,
                                block_timestamp,
                                db,
                                &mut factory,
                                &bundle,
                            )
                            .await;
                            count_burns += 1;
                        } else if topic == Swap::SIGNATURE_HASH.to_string()
                        {
                            handle_swap(
                                log,
                                block_timestamp,
                                db,
                                config,
                                &mut factory,
                                &bundle,
                            )
                            .await;
                            count_swaps += 1;
                        } else if topic == Sync::SIGNATURE_HASH.to_string()
                        {
                            handle_sync(
                                log,
                                db,
                                config,
                                &mut factory,
                                &mut bundle,
                            )
                            .await;
                            count_syncs += 1;
                        } else if topic
                            == Transfer::SIGNATURE_HASH.to_string()
                        {
                            handle_transfer(log, block_timestamp, db)
                                .await;
                            count_transfers += 1;
                        }
                    }
                    None => continue,
                }
            }

            tokio::join!(
                db.update_bundle(&bundle),
                db.update_factory(&factory)
            );

            info!("Procesed {} mints {} burns {} swaps {} sync and {} transfer events", count_mints, count_burns, count_swaps, count_syncs, count_transfers);
        }

        db.update_state(last_block).await;
    }
}
