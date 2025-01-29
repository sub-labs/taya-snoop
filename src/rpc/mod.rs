use std::str::FromStr;

use log::info;

use crate::{
    chains::Chain,
    configs::Config,
    db::models::{
        burn::DatabaseBurn,
        factory::PairCreated,
        log::DatabaseLog,
        mint::DatabaseMint,
        pair::{Burn, DatabasePair, Mint, Swap, Sync},
        swap::DatabaseSwap,
    },
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, Log},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::Filter,
    sol_types::SolEvent,
    transports::http::{Client, Http},
};

#[derive(Debug, Clone)]
pub struct Rpc {
    pub chain: Chain,
    pub client: RootProvider<Http<Client>>,
}

impl Rpc {
    pub async fn new(config: &Config) -> Self {
        info!("Starting rpc service");

        let client = ProviderBuilder::new()
            .on_http(config.rpc.clone().parse().unwrap());

        let client_id = client.get_chain_id().await;

        match client_id {
            Ok(value) => {
                if value != config.chain.id {
                    panic!("RPC chain id is invalid");
                }
            }
            Err(_) => panic!("unable to request eth_chainId"),
        }

        Self { chain: config.chain.clone(), client }
    }

    pub async fn get_last_block(&self) -> u64 {
        self.client
            .get_block_number()
            .await
            .expect("unable to get last block from RPC")
    }

    pub async fn get_factory_logs_batch(
        &self,
        first_block: u64,
        last_block: u64,
        config: &Config,
    ) -> (Vec<DatabaseLog>, Vec<DatabasePair>) {
        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block))
            .to_block(BlockNumberOrTag::Number(last_block))
            .address(config.factory.address)
            .event_signature(PairCreated::SIGNATURE_HASH);

        let logs = self
            .client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC")
            .into_iter();

        let mut db_logs: Vec<DatabaseLog> = vec![];
        let mut db_pairs_created: Vec<DatabasePair> = vec![];

        for log in logs {
            let database_log =
                DatabaseLog::from_rpc(&log, config.chain.id);

            let event = PairCreated::decode_log(&log.inner, true).unwrap();

            db_logs.push(database_log);

            let pair = DatabasePair::from_log(&log, event);

            db_pairs_created.push(pair);
        }

        (db_logs, db_pairs_created)
    }

    pub async fn get_pairs_logs_batch(
        &self,
        pairs: &[String],
        first_block: u64,
        last_block: u64,
        chain: &Chain,
    ) -> (
        Vec<DatabaseLog>,
        Vec<DatabaseMint>,
        Vec<DatabaseBurn>,
        Vec<DatabaseSwap>,
        Vec<Log<Sync>>,
    ) {
        let address_pairs: Vec<Address> = pairs
            .iter()
            .map(|pair| Address::from_str(pair).unwrap())
            .collect();

        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block))
            .to_block(BlockNumberOrTag::Number(last_block))
            .address(address_pairs)
            .events(vec![
                Mint::SIGNATURE_HASH,
                Burn::SIGNATURE_HASH,
                Swap::SIGNATURE_HASH,
                Sync::SIGNATURE_HASH,
            ]);

        let logs = self
            .client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC")
            .into_iter();

        let mut db_logs: Vec<DatabaseLog> = vec![];
        let mut db_mints: Vec<DatabaseMint> = vec![];
        let mut db_burns: Vec<DatabaseBurn> = vec![];
        let mut db_swaps: Vec<DatabaseSwap> = vec![];
        let mut sync_events: Vec<Log<Sync>> = vec![];

        for log in logs {
            println!("{:?}", log);
            match log.topic0() {
                Some(topic_raw) => {
                    let topic = topic_raw.to_string();

                    if topic == Mint::SIGNATURE {
                        let event: Log<Mint> =
                            Mint::decode_log(&log.inner, true).unwrap();

                        let db_mint = DatabaseMint::from_log(&log, event);

                        db_mints.push(db_mint);
                    }

                    if topic == Burn::SIGNATURE {
                        let event =
                            Burn::decode_log(&log.inner, true).unwrap();

                        let db_burn = DatabaseBurn::from_log(&log, event);

                        db_burns.push(db_burn)
                    }

                    if topic == Swap::SIGNATURE {
                        let event =
                            Swap::decode_log(&log.inner, true).unwrap();

                        let db_swap = DatabaseSwap::from_log(&log, event);

                        db_swaps.push(db_swap)
                    }

                    if topic == Sync::SIGNATURE {
                        let sync_event =
                            Sync::decode_log(&log.inner, true).unwrap();

                        sync_events.push(sync_event);
                    }

                    let db_log = DatabaseLog::from_rpc(&log, chain.id);

                    db_logs.push(db_log)
                }
                None => continue,
            }
        }

        (db_logs, db_mints, db_burns, db_swaps, sync_events)
    }
}
