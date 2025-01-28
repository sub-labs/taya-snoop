use alloy::sol;
use diesel::{
    prelude::{Insertable, Queryable},
    Selectable,
};
use field_count::FieldCount;

use crate::db::schema::pairs;

sol! {
    event PairCreated(address indexed token0, address indexed token1, address pair, uint);
    event Mint(address indexed sender, uint amount0, uint amount1);
    event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
    event Sync(uint112 reserve0, uint112 reserve1);
}

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = pairs)]
pub struct DatabasePairCreated {
    pub pair: String,
    pub token0: String,
    pub token1: String,
    pub index: i64,
}
