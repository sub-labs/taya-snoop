use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Chain {
    pub id: i64,
    pub name: &'static str,
}

pub const MAINNET: Chain = Chain { id: 1, name: "mainnet" };

pub static CHAINS: [Chain; 1] = [MAINNET];

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
