use std::str::FromStr;

use crate::chains::{get_chain, Chain};
use alloy::primitives::Address;
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
pub struct Factory {
    pub address: Address,
    pub start_block: i64,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub batch_size: usize,
    pub chain: Chain,
    pub db_url: String,
    pub debug: bool,
    pub rpc: String,
    pub factory: Factory,
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
            factory: Factory {
                address: Address::from_str(
                    "0xf4a772216e9266d062cee940b13a709f3542247b",
                )
                .unwrap(),
                start_block: 1864738,
            },
        }
    }
}
