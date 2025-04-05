pub mod models;
pub mod schema;

use std::collections::HashMap;

use crate::chains::Chain;

use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods,
    OptionalExtension, PgConnection, QueryDsl, QueryResult, RunQueryDsl,
};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use log::*;
use models::{
    bundle::DatabaseBundle,
    burn::DatabaseBurn,
    data::{
        DatabaseDexDayData, DatabasePairDayData, DatabasePairHourData,
        DatabaseTokenDayData,
    },
    factory::DatabaseFactory,
    mint::DatabaseMint,
    pair::DatabasePair,
    swap::DatabaseSwap,
    sync_state::DatabaseSyncState,
    token::DatabaseToken,
    transaction::DatabaseTransaction,
};

use schema::{
    bundles, burns, dex_day_data, factories, mints, pair_day_data,
    pair_hour_data, pairs, swaps, sync_state, token_day_data, tokens,
    transactions,
};

pub struct StorageCache {
    pub db: Database,
    pub factory: DatabaseFactory,
    pub bundle: DatabaseBundle,
    pub pairs: HashMap<String, DatabasePair>,
    pub tokens: HashMap<String, DatabaseToken>,
    pub transactions: HashMap<String, DatabaseTransaction>,
    pub mints: HashMap<String, DatabaseMint>,
    pub swaps: HashMap<String, DatabaseSwap>,
    pub burns: HashMap<String, DatabaseBurn>,
    pub pair_day_data: HashMap<String, DatabasePairDayData>,
    pub pair_hour_data: HashMap<String, DatabasePairHourData>,
    pub token_day_data: HashMap<String, DatabaseTokenDayData>,
    pub dex_day_data: HashMap<String, DatabaseDexDayData>,
}

impl StorageCache {
    pub async fn store(&self) {}
}

pub const MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("migrations/");

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db_url: String,
}

pub enum DatabaseKeys {
    State,
    Factory,
    Bundle,
    Logs,
    Burns,
    Mints,
    Swaps,
    Transactions,
    Users,
    Tokens,
    Pairs,
    FactoryDayData,
    PairHourData,
    PairDayData,
    TokenDayData,
}

impl DatabaseKeys {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseKeys::State => "sync_state",
            DatabaseKeys::Factory => "taya_swap",
            DatabaseKeys::Bundle => "bundle",
            DatabaseKeys::Logs => "logs",
            DatabaseKeys::Burns => "burns",
            DatabaseKeys::Mints => "mints",
            DatabaseKeys::Swaps => "swaps",
            DatabaseKeys::Transactions => "transactions",
            DatabaseKeys::Users => "users",
            DatabaseKeys::Tokens => "tokens",
            DatabaseKeys::Pairs => "pairs",
            DatabaseKeys::FactoryDayData => "dex_day_data",
            DatabaseKeys::PairHourData => "pair_hour_data",
            DatabaseKeys::PairDayData => "pair_day_data",
            DatabaseKeys::TokenDayData => "token_day_data",
        }
    }
}

impl Database {
    pub async fn new(db_url: String, chain: Chain) -> Self {
        info!("Starting database service");

        let mut db = PgConnection::establish(&db_url).unwrap();

        db.run_pending_migrations(MIGRATIONS).unwrap();

        Self { chain, db_url }
    }

    pub fn get_connection(&self) -> PgConnection {
        PgConnection::establish(&self.db_url)
            .expect("unable to connect to the database")
    }

    pub async fn get_last_block_indexed(&self) -> i32 {
        let mut connection = self.get_connection();

        let number: QueryResult<i32> = sync_state::dsl::sync_state
            .select(sync_state::dsl::last_block_indexed)
            .filter(sync_state::dsl::id.eq("sync_state"))
            .first::<i32>(&mut connection);

        match number {
            Ok(block) => block,
            Err(_) => {
                let new_sync_state = DatabaseSyncState::new();

                diesel::insert_into(sync_state::dsl::sync_state)
                    .values(new_sync_state.clone())
                    .execute(&mut connection)
                    .unwrap();

                new_sync_state.last_block_indexed
            }
        }
    }

    pub async fn get_factory(&self) -> DatabaseFactory {
        let mut connection = self.get_connection();

        match factories::dsl::factories
            .find(DatabaseKeys::Factory.as_str())
            .first::<DatabaseFactory>(&mut connection)
        {
            Ok(factory) => factory,
            Err(_) => {
                let new_factory = DatabaseFactory::new();

                diesel::insert_into(factories::dsl::factories)
                    .values(new_factory.clone())
                    .execute(&mut connection)
                    .unwrap();

                new_factory
            }
        }
    }

    pub async fn get_bundle(&self) -> DatabaseBundle {
        let mut connection: PgConnection = self.get_connection();

        match bundles::dsl::bundles
            .find(DatabaseKeys::Bundle.as_str())
            .first::<DatabaseBundle>(&mut connection)
        {
            Ok(bundle) => bundle,
            Err(_) => {
                let new_bundle = DatabaseBundle::new();

                diesel::insert_into(bundles::dsl::bundles)
                    .values(new_bundle.clone())
                    .execute(&mut connection)
                    .unwrap();

                new_bundle
            }
        }
    }

    pub async fn get_token(&self, id: &str) -> Option<DatabaseToken> {
        let mut connection: PgConnection = self.get_connection();

        tokens::dsl::tokens
            .find(id)
            .first::<DatabaseToken>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_pair(&self, id: &str) -> Option<DatabasePair> {
        let mut connection: PgConnection = self.get_connection();

        pairs::dsl::pairs
            .find(id)
            .first::<DatabasePair>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_pair_for_tokens(
        &self,
        token_a: &str,
        token_b: &str,
    ) -> Option<DatabasePair> {
        let mut connection = self.get_connection();

        pairs::dsl::pairs
            .filter(
                (pairs::dsl::token0
                    .eq(token_a)
                    .and(pairs::dsl::token1.eq(token_b)))
                .or(pairs::dsl::token0
                    .eq(token_b)
                    .and(pairs::dsl::token1.eq(token_a))),
            )
            .first::<DatabasePair>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_transaction(
        &self,
        id: &str,
    ) -> Option<DatabaseTransaction> {
        let mut connection: PgConnection = self.get_connection();

        transactions::dsl::transactions
            .find(id)
            .first::<DatabaseTransaction>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_mint(&self, id: &str) -> Option<DatabaseMint> {
        let mut connection: PgConnection = self.get_connection();

        mints::dsl::mints
            .find(id)
            .first::<DatabaseMint>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_burn(&self, id: &str) -> Option<DatabaseBurn> {
        let mut connection: PgConnection = self.get_connection();

        burns::dsl::burns
            .find(id)
            .first::<DatabaseBurn>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_dex_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseDexDayData> {
        let mut connection: PgConnection = self.get_connection();

        dex_day_data::dsl::dex_day_data
            .find(id)
            .first::<DatabaseDexDayData>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_pair_day_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairDayData> {
        let mut connection: PgConnection = self.get_connection();

        pair_day_data::dsl::pair_day_data
            .find(id)
            .first::<DatabasePairDayData>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_pair_hour_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairHourData> {
        let mut connection: PgConnection = self.get_connection();

        pair_hour_data::dsl::pair_hour_data
            .find(id)
            .first::<DatabasePairHourData>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn get_token_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseTokenDayData> {
        let mut connection: PgConnection = self.get_connection();

        token_day_data::dsl::token_day_data
            .find(id)
            .first::<DatabaseTokenDayData>(&mut connection)
            .optional()
            .unwrap()
    }

    pub async fn update_factory(&self, data: &DatabaseFactory) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(factories::dsl::factories)
            .values(data)
            .on_conflict(factories::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_token(&self, data: &DatabaseToken) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(tokens::dsl::tokens)
            .values(data)
            .on_conflict(tokens::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_pair(&self, data: &DatabasePair) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pairs::dsl::pairs)
            .values(data)
            .on_conflict(pairs::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_burn(&self, data: &DatabaseBurn) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(burns::dsl::burns)
            .values(data)
            .on_conflict(burns::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_mint(&self, data: &DatabaseMint) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(mints::dsl::mints)
            .values(data)
            .on_conflict(mints::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_bundle(&self, data: &DatabaseBundle) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(bundles::dsl::bundles)
            .values(data)
            .on_conflict(bundles::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_swap(&self, data: &DatabaseSwap) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(swaps::dsl::swaps)
            .values(data)
            .on_conflict(swaps::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_transaction(&self, data: &DatabaseTransaction) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(transactions::dsl::transactions)
            .values(data)
            .on_conflict(transactions::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_dex_day_data(&self, data: &DatabaseDexDayData) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(dex_day_data::dsl::dex_day_data)
            .values(data)
            .on_conflict(dex_day_data::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_pair_day_data(&self, data: &DatabasePairDayData) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pair_day_data::dsl::pair_day_data)
            .values(data)
            .on_conflict(pair_day_data::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_pair_hour_data(
        &self,
        data: &DatabasePairHourData,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pair_hour_data::dsl::pair_hour_data)
            .values(data)
            .on_conflict(pair_hour_data::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_token_day_data(
        &self,
        data: &DatabaseTokenDayData,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(token_day_data::dsl::token_day_data)
            .values(data)
            .on_conflict(token_day_data::id)
            .do_update()
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_state(&self, last_indexed_block: i32) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            sync_state::dsl::sync_state.find(DatabaseKeys::State.as_str()),
        )
        .set(sync_state::dsl::last_block_indexed.eq(last_indexed_block))
        .execute(&mut connection)
        .unwrap();
    }
}
