use fastnum::U256;
use serde::{Deserialize, Deserializer, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::str::FromStr;

pub fn parse_uint256(u: alloy::primitives::Uint<256, 4>) -> fastnum::U256 {
    let bytes: [u8; 32] = u.to_be_bytes();

    fastnum::U256::from_be_slice(&bytes).unwrap()
}

pub fn parse_uint112(u: alloy::primitives::Uint<112, 2>) -> fastnum::U256 {
    let bytes: [u8; 14] = u.to_be_bytes();

    fastnum::U256::from_be_slice(&bytes).unwrap()
}

pub struct SerU256(());

impl SerializeAs<U256> for SerU256 {
    fn serialize_as<S>(x: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = x.to_string();
        serializer.serialize_str(&s)
    }
}

impl<'de> DeserializeAs<'de, U256> for SerU256 {
    fn deserialize_as<D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        U256::from_str(&s).map_err(serde::de::Error::custom)
    }
}
