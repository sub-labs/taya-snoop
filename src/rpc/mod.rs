use std::str::FromStr;

use bigdecimal::BigDecimal;
use log::info;

use crate::{
    abi::{erc20::ERC20, factory::FACTORY},
    chains::Chain,
    configs::Config,
    handlers::{
        burn::Burn, mint::Mint, pairs::PairCreated, swap::Swap,
        sync::Sync, transfer::Transfer,
    },
    utils::format::parse_u256,
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::Address,
    providers::{DynProvider, Provider, ProviderBuilder},
    rpc::types::{Filter, Log},
    sol_types::SolEvent,
};

pub struct Rpc {
    pub chain: Chain,
    pub client: DynProvider,
}

impl Rpc {
    pub async fn new(config: &Config) -> Self {
        info!("Starting rpc service");

        let client = ProviderBuilder::new()
            .connect(&config.rpc.clone())
            .await
            .unwrap();

        let dyn_client = DynProvider::new(client.clone());

        let client_id = client.get_chain_id().await;

        match client_id {
            Ok(value) => {
                if value != config.chain.id {
                    panic!("RPC chain id is invalid");
                }
            }
            Err(_) => panic!("unable to request eth_chainId"),
        }

        Self { chain: config.chain.clone(), client: dyn_client }
    }

    pub async fn get_last_block(&self) -> Option<i32> {
        match self.client.get_block_number().await {
            Ok(block) => Some(block as i32),
            Err(_) => None,
        }
    }

    pub async fn get_factory_logs_batch(
        &self,
        first_block: u64,
        last_block: u64,
        config: &Config,
    ) -> Option<Vec<Log>> {
        let mut all_logs = Vec::new();

        let rpc_block_limit = 100;

        let mut current_block = first_block;
        while current_block <= last_block {
            let end_block = std::cmp::min(
                current_block + rpc_block_limit - 1,
                last_block,
            );

            let filter = Filter::new()
                .from_block(BlockNumberOrTag::Number(current_block))
                .to_block(BlockNumberOrTag::Number(end_block))
                .address(config.chain.factory.parse::<Address>().unwrap())
                .event(PairCreated::SIGNATURE);

            let batch_logs = self.client.get_logs(&filter).await.unwrap();

            all_logs.extend(batch_logs);

            current_block = end_block + 1;
        }

        Some(all_logs)
    }

    pub async fn get_pairs_logs_batch(
        &self,
        pairs: &[String],
        first_block: u64,
        last_block: u64,
    ) -> Option<Vec<Log>> {
        let address_pairs: Vec<Address> = pairs
            .iter()
            .map(|pair| Address::from_str(pair).unwrap())
            .collect();

        let mut all_logs = Vec::new();
        let rpc_block_limit = 100;

        let mut current_block = first_block;
        while current_block <= last_block {
            let end_block = std::cmp::min(
                current_block + rpc_block_limit - 1,
                last_block,
            );

            let filter = Filter::new()
                .from_block(BlockNumberOrTag::Number(current_block))
                .to_block(BlockNumberOrTag::Number(end_block))
                .address(address_pairs.clone())
                .events(vec![
                    Mint::SIGNATURE,
                    Burn::SIGNATURE,
                    Swap::SIGNATURE,
                    Sync::SIGNATURE,
                    Transfer::SIGNATURE,
                ]);

            let batch_logs = self.client.get_logs(&filter).await.unwrap();

            all_logs.extend(batch_logs);

            current_block = end_block + 1;
        }

        Some(all_logs)
    }

    pub async fn get_token_information(
        &self,
        token: String,
    ) -> (String, String, BigDecimal, i32) {
        let token_address = Address::from_str(&token).unwrap();
        let token = ERC20::new(token_address, &self.client);

        let multicall = self
            .client
            .multicall()
            .add(token.name())
            .add(token.symbol())
            .add(token.totalSupply())
            .add(token.decimals());

        let (name, symbol, total_supply, decimals) =
            multicall.aggregate().await.unwrap();

        (
            name._0,
            symbol._0,
            parse_u256(total_supply._0),
            decimals._0 as i32,
        )
    }

    pub async fn get_pair_for_tokens(
        &self,
        token0: String,
        token1: String,
        config: &Config,
    ) -> String {
        let factory = FACTORY::new(
            config.chain.factory.parse::<Address>().unwrap(),
            &self.client,
        );

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
