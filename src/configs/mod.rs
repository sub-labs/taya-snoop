use std::str::FromStr;

use crate::chains::{get_chain, Chain};
use alloy::{json_abi::Event, primitives::Address};
use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(name = "ETH Snoop", about = "ETH logs and event indexer.")]
pub struct IndexerArgs {
    #[arg(
        long,
        help = "Number of blocks to fetch in each batch of logs.",
        default_value_t = 50
    )]
    pub batch_size: usize,

    #[arg(
        long,
        help = "Specifies the target chain/network to index (e.g. 'mainnet', 'testnet').",
        default_value_t = String::from("mainnet")
    )]
    pub chain: String,

    #[arg(
        long,
        help = "PostgreSQL connection URL (e.g. 'postgres://user:password@host/dbname')."
    )]
    pub database: String,

    #[arg(
        long,
        help = "Enables verbose (debug-level) logging output.",
        default_value_t = false
    )]
    pub debug: bool,

    #[arg(
        long,
        help = "URL of the RPC endpoint to fetch chain data and logs."
    )]
    pub rpc: String,
}

#[derive(Debug, Clone)]
pub struct Subcontracts {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct ObserveContract {
    pub address: Address,
    pub events: Vec<Event>,
    pub is_factory: bool,
    pub start_block: i64,
    pub subcontracts: Option<Subcontracts>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub batch_size: usize,
    pub chain: Chain,
    pub db_url: String,
    pub debug: bool,
    pub rpc: String,
    pub contract: ObserveContract,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        let args = IndexerArgs::parse();

        let chain = get_chain(args.chain);

        Self {
            batch_size: args.batch_size,
            chain,
            db_url: args.database,
            debug: args.debug,
            rpc: args.rpc,
            contract: ObserveContract {
                // [Custom setup]: The address of the contract that emits events.
                address: Address::from_str("").unwrap(),
                // [Custom setup]: The list of events to index from the contract.
                events: vec![],
                // [Custom setup]: Indicates whether the contract is a factory that generates subcontracts.
                is_factory: false,
                // [Custom setup]: The block number from which to start syncing. Typically, this is the block where the contract was deployed.
                start_block: 0,
                // [Custom setup]: If the contract is a factory, specifies the events to listen for in the generated subcontracts.
                subcontracts: None,
            },
        }
    }
}
