use log::info;

use crate::{
    chains::Chain, configs::Config, db::models::log::DatabaseLog,
};
use alloy::{
    eips::BlockNumberOrTag,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::Filter,
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
                if value as i64 != config.chain.id {
                    panic!("RPC chain id is invalid");
                }
            }
            Err(_) => panic!("unable to request eth_chainId"),
        }

        Self { chain: config.chain.clone(), client }
    }

    pub async fn get_last_block(&self) -> i64 {
        self.client
            .get_block_number()
            .await
            .expect("unable to get last block from RPC") as i64
    }

    pub async fn get_logs_batch(
        &self,
        first_block: i64,
        last_block: i64,
        config: &Config,
    ) -> Vec<DatabaseLog> {
        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block as u64))
            .to_block(BlockNumberOrTag::Number(last_block as u64))
            .address(config.contract.address);

        self.client
            .get_logs(&filter)
            .await
            .expect("unable to get lgos from RPC")
            .into_iter()
            .map(|log| DatabaseLog::from_rpc(&log, config.chain.id))
            .collect()
    }
}
