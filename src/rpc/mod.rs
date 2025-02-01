use std::str::FromStr;

use fastnum::{u256, U256};
use log::info;

use crate::{
    abi::{erc20::ERC20, factory::FACTORY},
    chains::Chain,
    configs::Config,
    handlers::{
        burn::Burn, mint::Mint, pairs::PairCreated, swap::Swap,
        sync::Sync, transfer::Transfer,
    },
    utils::format::parse_uint256,
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{BlockTransactionsKind, Filter, Log},
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
            .event(PairCreated::SIGNATURE);

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
    ) -> Vec<Log> {
        let address_pairs: Vec<Address> = pairs
            .iter()
            .map(|pair| Address::from_str(pair).unwrap())
            .collect();

        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(first_block))
            .to_block(BlockNumberOrTag::Number(last_block))
            .address(address_pairs)
            .events(vec![
                Mint::SIGNATURE,
                Burn::SIGNATURE,
                Swap::SIGNATURE,
                Sync::SIGNATURE,
                Transfer::SIGNATURE,
            ]);

        self.client
            .get_logs(&filter)
            .await
            .expect("unable to get logs from RPC")
    }

    pub async fn get_token_information(
        &self,
        token: String,
    ) -> (String, String, U256, U256) {
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

        let total_supply: U256 = match token.totalSupply().call().await {
            Ok(total_supply) => parse_uint256(total_supply._0),
            Err(_) => u256!(0),
        };

        let decimals: U256 = match token.decimals().call().await {
            Ok(decimals) => U256::from(decimals._0),
            Err(_) => u256!(0),
        };

        (name, symbol, total_supply, decimals)
    }

    pub async fn get_block_timestamp(&self, block_number: i64) -> i64 {
        let block = self
            .client
            .get_block_by_number(
                BlockNumberOrTag::Number(block_number as u64),
                BlockTransactionsKind::Hashes,
            )
            .await
            .unwrap()
            .unwrap();

        block.header.timestamp as i64
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
            .to_lowercase()
    }
}
