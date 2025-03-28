use diesel::{AsChangeset, Insertable, Queryable};

use crate::db::{schema::sync_state, DatabaseKeys};

#[derive(Debug, Clone, Insertable, Queryable, AsChangeset)]
#[diesel(table_name = sync_state)]
#[diesel(check_for_backend(diesel::pg::Pg))]

pub struct DatabaseSyncState {
    pub id: String,
    pub last_block_indexed: i32,
}

impl Default for DatabaseSyncState {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseSyncState {
    pub fn new() -> Self {
        Self {
            id: DatabaseKeys::State.as_str().to_owned(),
            last_block_indexed: 0,
        }
    }
}
