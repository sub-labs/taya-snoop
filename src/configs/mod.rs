use crate::chains::{get_chain, Chain};
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
        help = "MongoDB connection URL (e.g. 'mongodb://user:password@host:27017/dbname')."
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
pub struct Config {
    pub batch_size: usize,
    pub chain: Chain,
    pub db_url: String,
    pub debug: bool,
    pub rpc: String,
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
        }
    }
}
