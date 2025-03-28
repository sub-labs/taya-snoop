use bigdecimal::BigDecimal;
use diesel::{AsChangeset, Insertable, Queryable};

use crate::{
    db::{schema::bundles, DatabaseKeys},
    utils::format::zero_bd,
};

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = bundles)]
#[diesel(check_for_backend(diesel::pg::Pg))]

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
