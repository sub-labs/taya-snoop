use std::str::FromStr;

use log::info;

use crate::{
    abi::erc20::ERC20,
    chains::Chain,
    configs::Config,
    db::models::{
        events::{DatabasePairCreated, PairCreated},
        log::DatabaseLog,
        tokens::DatabaseToken,
    },
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
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

    pub async fn get_factory_logs_batch(
        &self,
        first_block: i64,
        last_block: i64,
        config: &Config,
    ) -> (Vec<DatabaseLog>, Vec<DatabasePairCreated>) {
        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block as u64))
            .to_block(BlockNumberOrTag::Number(last_block as u64))
            .address(config.factory.address);

        let logs = self
            .client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC")
            .into_iter();

        let mut db_logs: Vec<DatabaseLog> = vec![];
        let mut db_pairs_created: Vec<DatabasePairCreated> = vec![];

        for log in logs {
            let database_log =
                DatabaseLog::from_rpc(&log, config.chain.id);

            let event = PairCreated::decode_log(&log.inner, true).unwrap();

            db_logs.push(database_log);
            db_pairs_created.push(DatabasePairCreated {
                pair: event.pair.to_string(),
                token0: event.token0.to_string(),
                token1: event.token1.to_string(),
                index: event._3.to_string().parse().unwrap(),
            });
        }

        (db_logs, db_pairs_created)
    }

    pub async fn get_token_information(
        &self,
        address: String,
    ) -> DatabaseToken {
        let token = ERC20::new(
            Address::from_str(&address).unwrap(),
            self.client.clone(),
        );

        let name = match token.name().call().await {
            Ok(name) => name._0,
            Err(_) => "UNKNOWN".to_string(),
        };

        let symbol = match token.symbol().call().await {
            Ok(symbol) => symbol._0,
            Err(_) => "UNKNOWN".to_string(),
        };

        let decimals = match token.decimals().call().await {
            Ok(decimals) => decimals._0 as i64,
            Err(_) => 0,
        };

        DatabaseToken { address, name, symbol, decimals }
    }
}
