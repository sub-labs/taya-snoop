use std::collections::HashMap;

use alloy::primitives::{address, Address};

#[derive(Debug, Clone)]
pub struct Chain {
    pub id: u64,
    pub name: &'static str,
    pub factory: Address,
    pub start_block: i64,
    pub weth: Address,
    pub whitelist_tokens: &'static [&'static str],
    pub dai_weth_pair: Option<Address>,
    pub usdc_weth_pair: Option<Address>,
    pub usdt_weth_pair: Option<Address>,
}

pub const TESTNET: Chain = Chain {
    id: 10143,
    name: "testnet",
    factory: address!("0xf4a772216e9266d062cee940b13a709f3542247b"),
    start_block: 4047383,
    weth: address!("0x760afe86e5de5fa0ee542fc7b7b713e1c5425701"),
    whitelist_tokens: &[
        "0x760afe86e5de5fa0ee542fc7b7b713e1c5425701", // WETH
        "0xddb9439df327910f9290a601f20bde775d856863", // tayUSDC
        "0xba7ccc60e4d15f3f71bff29771746e852eebeffe", // tayUSDT
    ],
    dai_weth_pair: None,
    usdc_weth_pair: Some(address!(
        "0x2ae000ddc4c47f542eb4a8b7d9213ac421877ab7"
    )),
    usdt_weth_pair: Some(address!(
        "0x8b52b43d2a90e79cd1cf1e204adb36df58658206"
    )),
};

pub static CHAINS: [Chain; 1] = [TESTNET];

pub fn get_chains() -> HashMap<String, Chain> {
    let mut chains: HashMap<String, Chain> = HashMap::new();

    for chain in CHAINS.iter() {
        chains.insert(chain.name.to_owned(), chain.to_owned());
    }

    chains
}

pub fn get_chain(chain: String) -> Chain {
    let chains = get_chains();

    let selected_chain = chains.get(&chain).expect("chain not found.");

    selected_chain.to_owned()
}
