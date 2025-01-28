use diesel::prelude::*;
use field_count::FieldCount;

use crate::db::schema::tokens;

#[derive(Selectable, Queryable, Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = tokens)]
pub struct DatabaseToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: i64,
}
