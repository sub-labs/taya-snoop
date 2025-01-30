use std::str::FromStr;

use log::info;

use crate::{
    abi::{erc20::ERC20, factory::FACTORY},
    chains::Chain,
    configs::Config,
    db::models::{
        factory::PairCreated,
        pair::{Burn, Mint, Swap, Sync, Transfer},
    },
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{Filter, Log},
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
    ) -> Vec<Log> {
        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block))
            .to_block(BlockNumberOrTag::Number(last_block))
            .address(config.factory.address)
            .event_signature(PairCreated::SIGNATURE_HASH);

        self.client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC")
    }

    pub async fn get_pairs_logs_batch(
        &self,
        pairs: &[String],
        first_block: u64,
        last_block: u64,
    ) -> (Vec<Log>, Vec<Log>, Vec<Log>, Vec<Log>, Vec<Log>) {
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
                Transfer::SIGNATURE_HASH,
            ]);

        let logs = self
            .client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC");

        let mut mint_logs: Vec<Log> = vec![];
        let mut burn_logs: Vec<Log> = vec![];
        let mut swap_logs: Vec<Log> = vec![];
        let mut sync_logs: Vec<Log> = vec![];
        let mut transfer_logs: Vec<Log> = vec![];

        for log in logs {
            match log.topic0() {
                Some(topic_raw) => {
                    let topic = topic_raw.to_string();

                    if topic == Mint::SIGNATURE {
                        mint_logs.push(log);
                    } else if topic == Burn::SIGNATURE {
                        burn_logs.push(log)
                    } else if topic == Swap::SIGNATURE {
                        swap_logs.push(log)
                    } else if topic == Sync::SIGNATURE {
                        sync_logs.push(log);
                    } else if topic == Transfer::SIGNATURE {
                        transfer_logs.push(log);
                    }
                }
                None => continue,
            }
        }

        (mint_logs, burn_logs, swap_logs, sync_logs, transfer_logs)
    }

    pub async fn get_token_information(
        &self,
        token: String,
    ) -> (String, String, String, i64) {
        let token =
            ERC20::new(Address::from_str(&token).unwrap(), &self.client);

        let name = match token.name().call().await {
            Ok(name) => name._0,
            Err(_) => "UNKNOWN".to_owned(),
        };

        let symbol = match token.symbol().call().await {
            Ok(symbol) => symbol._0,
            Err(_) => "UNKNOWN".to_owned(),
        };

        let total_supply = match token.totalSupply().call().await {
            Ok(total_supply) => total_supply._0.to_string(),
            Err(_) => "0".to_owned(),
        };

        let decimals = match token.decimals().call().await {
            Ok(decimals) => decimals._0,
            Err(_) => 0,
        };

        (name, symbol, total_supply, decimals as i64)
    }

    pub async fn get_pair_for_tokens(
        &self,
        token0: String,
        token1: String,
        config: &Config,
    ) -> String {
        let factory = FACTORY::new(config.factory.address, &self.client);

        factory
            .getPair(
                Address::from_str(&token0).unwrap(),
                Address::from_str(&token1).unwrap(),
            )
            .call()
            .await
            .unwrap()
            ._0
            .to_string()
    }
}
