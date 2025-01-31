use crate::{
    configs::Config,
    db::{models::token::DatabaseToken, Database},
    rpc::Rpc,
};
use alloy::{primitives::Address, rpc::types::Log};
use fastnum::{decimal::Context, u256, udec256, U256, UD256};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

pub const MINIMUM_USD_THRESHOLD_NEW_PAIRS: UD256 = udec256!(400000);

pub const MINIMUM_LIQUIDITY_THRESHOLD_ETH: UD256 = udec256!(2);

pub const WHITELIST_TOKENS: [&str; 4] = [
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
    "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
    "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
];

pub const WETH_ADDRESS: &str =
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
pub const DAI_WETH_PAIR: &str =
    "0xa478c2975ab1ea89e8196811f51a7b7ade33eb11";
pub const USDC_WETH_PAIR: &str =
    "0xb4e16d0168e52d35cacd2c6185b44281ec28c9dc";
pub const USDT_WETH_PAIR: &str =
    "0x0d4a11d5eeaac28ec3f61d100daf4d40471f1852";

pub async fn get_eth_price_usd(db: &Database) -> UD256 {
    let dai_pair = db.get_pair(DAI_WETH_PAIR.to_owned()).await;
    let usdc_pair = db.get_pair(USDC_WETH_PAIR.to_owned()).await;
    let usdt_pair = db.get_pair(USDT_WETH_PAIR.to_owned()).await;

    match (dai_pair, usdc_pair, usdt_pair) {
        (Some(dai), Some(usdc), Some(usdt)) => {
            let total_liquidity_eth =
                dai.reserve1.add(usdt.reserve0).add(usdc.reserve1);

            if total_liquidity_eth.eq(&udec256!(0)) {
                return udec256!(0);
            }

            let dai_weight = dai.reserve1.div(total_liquidity_eth);
            let usdc_weight = usdc.reserve1.div(total_liquidity_eth);
            let usdt_weight = usdt.reserve0.div(total_liquidity_eth);

            dai.token0_price
                .mul(dai_weight)
                .add(usdc.token0_price.mul(usdc_weight))
                .add(usdt.token1_price.mul(usdt_weight))
        }
        (Some(dai), Some(usdc), None) => {
            let total_liquidity_eth = dai.reserve1.add(usdc.reserve1);

            if total_liquidity_eth.eq(&udec256!(0)) {
                return udec256!(0);
            }

            let dai_weight = dai.reserve1.div(total_liquidity_eth);
            let usdc_weight = usdc.reserve1.div(total_liquidity_eth);

            dai.token0_price
                .mul(dai_weight)
                .add(usdc.token0_price.mul(usdc_weight))
        }
        (_, Some(usdc), _) => usdc.token0_price,
        // No pairs
        _ => udec256!(0),
    }
}

pub async fn find_eth_per_token(
    token: &DatabaseToken,
    rpc: &Rpc,
    db: &Database,
    config: &Config,
) -> UD256 {
    if token.id == WETH_ADDRESS {
        return udec256!(0);
    }

    // Loop through a set of whitelisted tokens to check if there is any pair for this token.
    for whitelist_token in WHITELIST_TOKENS {
        let pair_address = rpc
            .get_pair_for_tokens(
                token.id.clone(),
                whitelist_token.to_owned(),
                config,
            )
            .await;

        if pair_address != Address::ZERO.to_string() {
            let pair = db.get_pair(pair_address).await;
            if pair.is_none() {
                continue;
            }

            let pair = pair.unwrap();

            if pair.token0 == token.id
                && pair.reserve_eth.ge(&MINIMUM_LIQUIDITY_THRESHOLD_ETH)
            {
                let token0 = db.get_token(pair.token0).await;
                if token0.is_none() {
                    continue;
                }

                let token0 = token0.unwrap();

                return pair.token0_price.mul(token0.derived_eth);
            }
            if pair.token1 == token.id
                && pair.reserve_eth.ge(&MINIMUM_LIQUIDITY_THRESHOLD_ETH)
            {
                let token1 = db.get_token(pair.token1).await;
                if token1.is_none() {
                    continue;
                }

                let token1 = token1.unwrap();

                return pair.token1_price.mul(token1.derived_eth);
            }
        }
    }

    udec256!(0)
}

pub async fn get_tracked_liquidity_usd(
    token_amount0: UD256,
    token0: &DatabaseToken,
    token_amount1: UD256,
    token1: &DatabaseToken,
    db: &Database,
) -> UD256 {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.mul(bundle.eth_price);
    let price1 = token1.derived_eth.mul(bundle.eth_price);

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0.mul(price0).add(token_amount1.mul(price1));
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount0.mul(price0).mul(udec256!(2));
    }

    if !WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return token_amount1.mul(price1).mul(udec256!(2));
    }

    udec256!(0)
}

pub async fn update_pair_day_data(log: &Log) {}

pub async fn update_pair_hour_data(log: &Log) {}

pub async fn update_factory_day_data(log: &Log) {}

pub async fn update_token_day_data(token: &DatabaseToken, log: &Log) {}

fn exponent_to_bigdecimal(decimals: &U256) -> UD256 {
    let mut bd = u256!(1);

    let mut i = u256!(0);
    while i < *decimals {
        bd = bd.mul(u256!(10));
        i += &u256!(1);
    }

    UD256::from_str(&bd.to_string(), Context::default()).unwrap()
}

pub fn convert_token_to_decimal(
    token_amount: &U256,
    exchange_decimals: &U256,
) -> UD256 {
    let token_amount_decimal =
        UD256::from_str(&token_amount.to_string(), Context::default())
            .unwrap();

    if exchange_decimals.is_zero() {
        return token_amount_decimal;
    } else {
        let divisor = exponent_to_bigdecimal(exchange_decimals);
        token_amount_decimal / divisor
    }
}

pub fn parse_uint256(u: alloy::primitives::Uint<256, 4>) -> fastnum::U256 {
    let bytes: [u8; 32] = u.to_be_bytes();

    fastnum::U256::from_be_slice(&bytes).unwrap()
}

pub fn parse_uint112(u: alloy::primitives::Uint<112, 2>) -> fastnum::U256 {
    let bytes: [u8; 32] = u.to_be_bytes();

    fastnum::U256::from_be_slice(&bytes).unwrap()
}

/// Serialize `U256` as a string
pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// Deserialize `U256` from a string
pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<U256>().map_err(serde::de::Error::custom)
}
