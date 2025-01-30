use alloy::{primitives::Address, rpc::types::Log};
use bigdecimal::{BigDecimal, FromPrimitive, One, Zero};

use crate::{
    configs::Config,
    db::{models::token::DatabaseToken, Database},
    rpc::Rpc,
};

pub const MINIMUM_LIQUIDITY_THRESHOLD_ETH: f64 = 2.0;

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

pub async fn get_eth_price_usd(db: &Database) -> BigDecimal {
    let dai_pair = db.get_pair(DAI_WETH_PAIR.to_owned()).await;
    let usdc_pair = db.get_pair(USDC_WETH_PAIR.to_owned()).await;
    let usdt_pair = db.get_pair(USDT_WETH_PAIR.to_owned()).await;

    match (dai_pair, usdc_pair, usdt_pair) {
        (Some(dai), Some(usdc), Some(usdt)) => {
            let total_liquidity_eth = dai.reserve1.clone()
                + usdc.reserve1.clone()
                + usdt.reserve0.clone();

            if total_liquidity_eth.eq(&BigDecimal::zero()) {
                return BigDecimal::zero();
            }

            let dai_weight =
                dai.reserve1.clone() / total_liquidity_eth.clone();
            let usdc_weight =
                usdc.reserve1.clone() / total_liquidity_eth.clone();
            let usdt_weight =
                usdt.reserve0.clone() / total_liquidity_eth.clone();

            (dai.token0_price * dai_weight)
                + (usdc.token0_price * usdc_weight)
                + (usdt.token1_price * usdt_weight)
        }
        (Some(dai), Some(usdc), None) => {
            let total_liquidity_eth =
                dai.reserve1.clone() + usdc.reserve1.clone();

            if total_liquidity_eth.eq(&BigDecimal::zero()) {
                return BigDecimal::zero();
            }

            let dai_weight =
                dai.reserve1.clone() / total_liquidity_eth.clone();
            let usdc_weight =
                usdc.reserve1.clone() / total_liquidity_eth.clone();

            (dai.token0_price * dai_weight)
                + (usdc.token0_price * usdc_weight)
        }
        (_, Some(usdc), _) => usdc.token0_price,
        // No pairs
        _ => BigDecimal::zero(),
    }
}

pub async fn find_eth_per_token(
    token: &DatabaseToken,
    rpc: &Rpc,
    db: &Database,
    config: &Config,
) -> BigDecimal {
    if token.id == WETH_ADDRESS {
        return BigDecimal::one();
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
            let minimum_liquidity =
                BigDecimal::from_f64(MINIMUM_LIQUIDITY_THRESHOLD_ETH)
                    .unwrap();

            if pair.token0 == token.id
                && pair.reserve_eth.ge(&minimum_liquidity)
            {
                let token0 = db.get_token(pair.token0).await;
                if token0.is_none() {
                    continue;
                }

                let token0 = token0.unwrap();

                return pair.token0_price * token0.derived_eth;
            }
            if pair.token1 == token.id
                && pair.reserve_eth.ge(&minimum_liquidity)
            {
                let token1 = db.get_token(pair.token1).await;
                if token1.is_none() {
                    continue;
                }

                let token1 = token1.unwrap();

                return pair.token1_price * token1.derived_eth;
            }
        }
    }

    BigDecimal::zero()
}

pub async fn get_tracked_liquidity_usd(
    token_amount0: BigDecimal,
    token0: &DatabaseToken,
    token_amount1: BigDecimal,
    token1: &DatabaseToken,
    db: &Database,
) -> BigDecimal {
    let bundle = db.get_bundle().await;

    let price0 = token0.derived_eth.clone() * bundle.eth_price.clone();
    let price1 = token1.derived_eth.clone() * bundle.eth_price.clone();

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount0 * price0) + (token_amount1 * price1);
    }

    if WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount0 * price0)
            * BigDecimal::from_usize(2).unwrap();
    }

    if !WHITELIST_TOKENS.contains(&token0.id.as_str())
        && !WHITELIST_TOKENS.contains(&token1.id.as_str())
    {
        return (token_amount1 * price1)
            * BigDecimal::from_usize(2).unwrap();
    }

    BigDecimal::zero()
}

pub async fn update_pair_day_data(log: &Log) {}

pub async fn update_pair_hour_data(log: &Log) {}

pub async fn update_factory_day_data(log: &Log) {}

pub async fn update_token_day_data(token: &DatabaseToken, log: &Log) {}
