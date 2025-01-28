use log::info;

use crate::{
    chains::Chain,
    configs::Config,
    db::models::{
        factory::PairCreated, log::DatabaseLog, pair::DatabasePair,
    },
};
use alloy::{
    eips::BlockNumberOrTag,
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
            .address(config.factory.address);

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

            let pair = DatabasePair::new(
                event.pair.to_string(),
                event.token0.to_string(),
                event.token1.to_string(),
            );

            db_pairs_created.push(pair);
        }

        (db_logs, db_pairs_created)
    }
}
