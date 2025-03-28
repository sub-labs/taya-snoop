pub mod models;
pub mod schema;

use crate::chains::Chain;

use diesel::{
    Connection, ExpressionMethods, OptionalExtension, PgConnection,
    QueryDsl, QueryResult, RunQueryDsl,
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
    token::DatabaseToken,
    transaction::DatabaseTransaction,
};

use schema::{
    bundles, burns, dex_day_data, factories, mints, pair_day_data,
    pair_hour_data, pairs, swaps, sync_state, token_day_data, tokens,
    transactions,
};

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

static DATABASE: &str = "indexer";

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
            Err(_) => panic!("unable to get last synced block"),
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

        return tokens::dsl::tokens
            .find(id)
            .first::<DatabaseToken>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_pair(&self, id: &str) -> Option<DatabasePair> {
        let mut connection: PgConnection = self.get_connection();

        return pairs::dsl::pairs
            .find(id)
            .first::<DatabasePair>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_transaction(
        &self,
        id: &str,
    ) -> Option<DatabaseTransaction> {
        let mut connection: PgConnection = self.get_connection();

        return transactions::dsl::transactions
            .find(id)
            .first::<DatabaseTransaction>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_mint(&self, id: &str) -> Option<DatabaseMint> {
        let mut connection: PgConnection = self.get_connection();

        return mints::dsl::mints
            .find(id)
            .first::<DatabaseMint>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_burn(&self, id: &str) -> Option<DatabaseBurn> {
        let mut connection: PgConnection = self.get_connection();

        return burns::dsl::burns
            .find(id)
            .first::<DatabaseBurn>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_dex_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseDexDayData> {
        let mut connection: PgConnection = self.get_connection();

        return dex_day_data::dsl::dex_day_data
            .find(id)
            .first::<DatabaseDexDayData>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_pair_day_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairDayData> {
        let mut connection: PgConnection = self.get_connection();

        return pair_day_data::dsl::pair_day_data
            .find(id)
            .first::<DatabasePairDayData>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_pair_hour_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairHourData> {
        let mut connection: PgConnection = self.get_connection();

        return pair_hour_data::dsl::pair_hour_data
            .find(id)
            .first::<DatabasePairHourData>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn get_token_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseTokenDayData> {
        let mut connection: PgConnection = self.get_connection();

        return token_day_data::dsl::token_day_data
            .find(id)
            .first::<DatabaseTokenDayData>(&mut connection)
            .optional()
            .unwrap();
    }

    pub async fn update_factory(&self, data: &DatabaseFactory) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            factories::dsl::factories.find(DatabaseKeys::Factory.as_str()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_token(&self, data: &DatabaseToken) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(tokens::dsl::tokens.find(data.id.clone()))
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_pair(&self, data: &DatabasePair) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(pairs::dsl::pairs.find(data.id.clone()))
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_burn(&self, data: &DatabaseBurn) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(burns::dsl::burns.find(data.id.clone()))
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_mint(&self, data: &DatabaseMint) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(mints::dsl::mints.find(data.id.clone()))
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_bundle(&self, data: &DatabaseBundle) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            bundles::dsl::bundles.find(DatabaseKeys::Bundle.as_str()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_swap(&self, data: &DatabaseSwap) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(swaps::dsl::swaps.find(data.id.clone()))
            .set(data)
            .execute(&mut connection)
            .unwrap();
    }

    pub async fn update_transaction(&self, data: &DatabaseTransaction) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            transactions::dsl::transactions.find(data.id.clone()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_dex_day_data(&self, data: &DatabaseDexDayData) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            dex_day_data::dsl::dex_day_data.find(data.id.clone()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_pair_day_data(&self, data: &DatabasePairDayData) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            pair_day_data::dsl::pair_day_data.find(data.id.clone()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_pair_hour_data(
        &self,
        data: &DatabasePairHourData,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            pair_hour_data::dsl::pair_hour_data.find(data.id.clone()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }

    pub async fn update_token_day_data(
        &self,
        data: &DatabaseTokenDayData,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::update(
            token_day_data::dsl::token_day_data.find(data.id.clone()),
        )
        .set(data)
        .execute(&mut connection)
        .unwrap();
    }
}
