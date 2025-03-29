use diesel::{AsChangeset, Insertable, Queryable};

use crate::db::schema::transactions;

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DatabaseTransaction {
    pub id: String,
    pub block_number: i32,
    pub timestamp: i32,
    pub mints: Vec<Option<String>>,
    pub swaps: Vec<Option<String>>,
    pub burns: Vec<Option<String>>,
}

impl DatabaseTransaction {
    pub fn new(hash: String, block_number: i32, timestamp: i32) -> Self {
        Self {
            id: hash,
            block_number,
            timestamp,
            mints: Vec::new(),
            burns: Vec::new(),
            swaps: Vec::new(),
        }
    }
}
