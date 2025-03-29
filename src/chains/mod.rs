use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chain {
    pub id: u64,
    pub name: &'static str,
    pub factory: &'static str,
    pub start_block: i32,
    pub weth: &'static str,
    pub whitelist_tokens: &'static [&'static str],
    pub usdc_weth_pair: Option<&'static str>,
    pub usdt_weth_pair: Option<&'static str>,
    pub minimum_usd_threshold_new_pairs: i32,
    pub minimum_liquidity_threshold_eth: i32,
}

pub const TESTNET: Chain = Chain {
    id: 10143,
    name: "testnet",
    factory: "0xf3fd5503fb2bb5f5a7ae713e621ac5c50f191fb3",
    start_block: 5253609,
    weth: "0x760afe86e5de5fa0ee542fc7b7b713e1c5425701",
    whitelist_tokens: &[
        "0x760afe86e5de5fa0ee542fc7b7b713e1c5425701", // WMON
        "0xf817257fed379853cde0fa4f97ab987181b1e5ea", // USDC
        "0x88b8e2161dedc77ef4ab7585569d2415a1c1055d", // USDT
    ],
    usdc_weth_pair: Some("0x1512cb2431b9b14ed14e39dad75496b922481cfd"),
    usdt_weth_pair: Some("0x488e1d7f4ac40ff42817efbdb5db36508277dc99"),
    minimum_usd_threshold_new_pairs: 10000,
    minimum_liquidity_threshold_eth: 2,
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
