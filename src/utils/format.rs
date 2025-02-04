use fastnum::decimal::Context;

pub fn parse_ud256(u: alloy::primitives::Uint<256, 4>) -> fastnum::UD256 {
    fastnum::UD256::from_str(&u.to_string(), Context::default()).unwrap()
}

pub fn parse_ud112(u: alloy::primitives::Uint<112, 2>) -> fastnum::UD256 {
    fastnum::UD256::from_str(&u.to_string(), Context::default()).unwrap()
}
