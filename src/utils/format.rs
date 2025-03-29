use std::str::FromStr;

use alloy::primitives::Address;
use bigdecimal::BigDecimal;

fn exponent_to_big_decimal(decimals: i32) -> BigDecimal {
    let mut bd = one_bd();

    let mut i = 0;
    while i < decimals {
        bd *= 10;
        i += 1;
    }

    bd
}

pub fn convert_token_to_decimal(
    token_amount: &BigDecimal,
    decimals: i32,
) -> BigDecimal {
    if decimals == 0 {
        token_amount.clone()
    } else {
        let divisor = exponent_to_big_decimal(decimals);
        token_amount / divisor
    }
}

pub fn parse_u256(u: alloy::primitives::Uint<256, 4>) -> BigDecimal {
    BigDecimal::from_str(&u.to_string()).unwrap()
}

pub fn parse_u112(u: alloy::primitives::Uint<112, 2>) -> BigDecimal {
    BigDecimal::from_str(&u.to_string()).unwrap()
}

pub fn zero_bd() -> BigDecimal {
    BigDecimal::from(0)
}

pub fn one_bd() -> BigDecimal {
    BigDecimal::from(1)
}

pub fn address_zero() -> String {
    Address::ZERO.to_string().to_lowercase()
}
