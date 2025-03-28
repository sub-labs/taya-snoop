use diesel::{AsChangeset, Insertable, Queryable};

use crate::db::schema::transactions;

#[derive(Queryable, Insertable, Debug, Clone, AsChangeset)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseTransaction {
    pub id: String,
    pub block_number: i32,
    pub timestamp: i32,
}
