use diesel::{Insertable, Queryable};

use crate::db::schema::transactions;

#[derive(Queryable, Insertable, Debug, Clone)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: String,
    pub block_number: i32,
    pub timestamp: i32,
}
