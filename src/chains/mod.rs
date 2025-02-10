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
    start_block: 1864738,
    weth: address!("0x760afe86e5de5fa0ee542fc7b7b713e1c5425701"),
    whitelist_tokens: &[
        "0x760afe86e5de5fa0ee542fc7b7b713e1c5425701",
        "0x1ed9ca7e442a91591acecfb2d40e843e4fee00ff",
        "0xff901f49b8864ad60cc5799cc9172ae0455ec1d3",
        "0x2f1014530ed895245ecb5f9a79de023102f2e741",
    ],
    dai_weth_pair: Some(address!(
        "0x750152d4631cd5f06c1fd7c0bc935aa92b7adc2b"
    )),
    usdc_weth_pair: Some(address!(
        "0x66367136ba1b3917f86aab7953839102a2428b2b"
    )),
    usdt_weth_pair: Some(address!(
        "0xec1d5bbc9498115408a78a3f65a9188326b235af"
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
