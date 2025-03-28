use bigdecimal::BigDecimal;
use diesel::{Insertable, Queryable};

use crate::{
    db::{schema::bundles, DatabaseKeys},
    utils::format::zero_bd,
};

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = bundles)]
pub struct DatabaseBundle {
    pub id: String,
    pub eth_price: BigDecimal,
}

impl Default for DatabaseBundle {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseBundle {
    pub fn new() -> Self {
        Self {
            id: DatabaseKeys::Bundle.as_str().to_owned(),
            eth_price: zero_bd(),
        }
    }
}
