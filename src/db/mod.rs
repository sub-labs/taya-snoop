pub mod models;
pub mod schema;

use self::schema::logs;
use crate::chains::Chain;
use core::panic;
use diesel::{
    Connection, ExpressionMethods, PgConnection, QueryDsl, QueryResult,
    RunQueryDsl,
};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use field_count::FieldCount;
use futures::future::join_all;
use log::*;
use models::log::DatabaseLog;
use schema::sync_state::{self, id, last_block_number};
use std::cmp::min;

pub const MAX_PARAM_SIZE: u16 = u16::MAX;

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("migrations/");

pub enum DatabaseTables {
    SyncState,
    Events,
    Logs,
}

impl DatabaseTables {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseTables::SyncState => "sync_state",
            DatabaseTables::Events => "events",
            DatabaseTables::Logs => "logs",
        }
    }
}

pub struct StoreData {
    pub logs: Vec<DatabaseLog>,
}

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db_url: String,
}

impl Database {
    pub async fn new(db_url: String, chain: Chain) -> Self {
        info!("Starting database service");

        let mut db = PgConnection::establish(&db_url)
            .expect("unable to connect to the database");

        db.run_pending_migrations(MIGRATIONS).unwrap();

        Self { chain, db_url }
    }

    pub fn get_connection(&self) -> PgConnection {
        PgConnection::establish(&self.db_url)
            .expect("unable to connect to the database")
    }

    pub fn get_last_block_indexed(&self) -> i64 {
        let mut connection = self.get_connection();

        let number: QueryResult<i64> = sync_state::dsl::sync_state
            .select(sync_state::dsl::last_block_number)
            .filter(id.eq("sync_state"))
            .first::<i64>(&mut connection);

        match number {
            Ok(block) => block,
            Err(_) => panic!("unable to get last synced block"),
        }
    }

    pub fn update_last_block_indexed(&self, new_last_block_number: i64) {
        let mut connection = self.get_connection();

        diesel::update(
            sync_state::dsl::sync_state.filter(id.eq("sync_state")),
        )
        .set(last_block_number.eq(&new_last_block_number))
        .execute(&mut connection)
        .expect("unable to update sync state into database");
    }

    async fn store_logs(&self, logs: &[DatabaseLog]) {
        let mut connection = self.get_connection();

        let chunks = get_chunks(logs.len(), DatabaseLog::field_count());

        for (start, end) in chunks {
            diesel::insert_into(logs::dsl::logs)
                .values(&logs[start..end])
                .on_conflict_do_nothing()
                .execute(&mut connection)
                .expect("unable to store logs into database");
        }
    }

    pub async fn store_data(&self, data: StoreData) {
        let mut stores: Vec<tokio::task::JoinHandle<()>> = vec![];

        if !data.logs.is_empty() {
            let work = tokio::spawn({
                let logs: Vec<DatabaseLog> = data.logs.clone();

                let db = self.clone();

                async move { db.store_logs(&logs).await }
            });

            stores.push(work);
        }

        let res = join_all(stores).await;

        let errored: Vec<_> =
            res.iter().filter(|res| res.is_err()).collect();

        if !errored.is_empty() {
            panic!("failed to store all chain primitive elements")
        }

        info!("Inserted: logs ({}).", data.logs.len());
    }
}

/// Ref: https://github.com/aptos-labs/aptos-core/blob/main/crates/indexer/src/database.rs#L32
/// Given diesel has a limit of how many parameters can be inserted in a single operation (u16::MAX)
/// we may need to chunk an array of items based on how many columns are in the table.
/// This function returns boundaries of chunks in the form of (start_index, end_index)
pub fn get_chunks(
    num_items_to_insert: usize,
    column_count: usize,
) -> Vec<(usize, usize)> {
    let max_item_size = MAX_PARAM_SIZE as usize / column_count;
    let mut chunk: (usize, usize) =
        (0, min(num_items_to_insert, max_item_size));
    let mut chunks = vec![chunk];
    while chunk.1 != num_items_to_insert {
        chunk = (
            chunk.0 + max_item_size,
            min(num_items_to_insert, chunk.1 + max_item_size),
        );
        chunks.push(chunk);
    }
    chunks
}
