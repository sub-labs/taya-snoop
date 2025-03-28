pub mod models;
pub mod schema;

use crate::chains::Chain;

use diesel::{
    Connection, ExpressionMethods, PgConnection, QueryDsl, QueryResult,
    RunQueryDsl,
};
use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
use log::*;
use models::{
    bundle::DatabaseBundle,
    burn::DatabaseBurn,
    data::{
        DatabaseFactoryDayData, DatabasePairDayData, DatabasePairHourData,
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
use mongodb::bson::{doc, to_document};
use schema::sync_state::{self, id, last_block_indexed};
use serde::{de::DeserializeOwned, Serialize};

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
            .filter(id.eq("sync_state"))
            .first::<i32>(&mut connection);

        match number {
            Ok(block) => block,
            Err(_) => panic!("unable to get last synced block"),
        }
    }

    pub async fn get_factory(&self) -> DatabaseFactory {
        let factory_key = DatabaseKeys::Factory.as_str();

        match self
            .db
            .collection::<DatabaseFactory>(factory_key)
            .find_one(doc! { "id": { "$eq": factory_key}})
            .await
            .unwrap()
        {
            Some(factory) => factory,
            None => {
                let new_factory = DatabaseFactory::new();

                self.db
                    .collection::<DatabaseFactory>(factory_key)
                    .insert_one(&new_factory)
                    .await
                    .unwrap();

                new_factory
            }
        }
    }

    pub async fn get_bundle(&self) -> DatabaseBundle {
        let bundle_key = DatabaseKeys::Bundle.as_str();

        match self
            .db
            .collection::<DatabaseBundle>(bundle_key)
            .find_one(doc! { "id": { "$eq": bundle_key}})
            .await
            .unwrap()
        {
            Some(bundle) => bundle,
            None => {
                let new_bundle: DatabaseBundle = DatabaseBundle::new();

                self.db
                    .collection::<DatabaseBundle>(bundle_key)
                    .insert_one(&new_bundle)
                    .await
                    .unwrap();

                new_bundle
            }
        }
    }

    pub async fn get_token(&self, id: &str) -> Option<DatabaseToken> {
        return self.get::<DatabaseToken>(DatabaseKeys::Tokens, id).await;
    }

    pub async fn get_pair(&self, id: &str) -> Option<DatabasePair> {
        return self.get::<DatabasePair>(DatabaseKeys::Pairs, id).await;
    }

    pub async fn get_transaction(
        &self,
        id: &str,
    ) -> Option<DatabaseTransaction> {
        return self
            .get::<DatabaseTransaction>(DatabaseKeys::Transactions, id)
            .await;
    }

    pub async fn get_mint(&self, id: &str) -> Option<DatabaseMint> {
        return self.get::<DatabaseMint>(DatabaseKeys::Mints, id).await;
    }

    pub async fn get_burn(&self, id: &str) -> Option<DatabaseBurn> {
        return self.get::<DatabaseBurn>(DatabaseKeys::Burns, id).await;
    }

    pub async fn get_factory_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseFactoryDayData> {
        return self
            .get::<DatabaseFactoryDayData>(
                DatabaseKeys::FactoryDayData,
                id,
            )
            .await;
    }

    pub async fn get_pair_day_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairDayData> {
        return self
            .get::<DatabasePairDayData>(DatabaseKeys::PairDayData, id)
            .await;
    }

    pub async fn get_pair_hour_data(
        &self,
        id: &str,
    ) -> Option<DatabasePairHourData> {
        return self
            .get::<DatabasePairHourData>(DatabaseKeys::PairHourData, id)
            .await;
    }

    pub async fn get_token_day_data(
        &self,
        id: &str,
    ) -> Option<DatabaseTokenDayData> {
        return self
            .get::<DatabaseTokenDayData>(DatabaseKeys::TokenDayData, id)
            .await;
    }

    async fn get<T>(&self, collection: DatabaseKeys, id: &str) -> Option<T>
    where
        T: DeserializeOwned + Unpin + Send + Sync + 'static,
    {
        self.db
            .collection::<T>(collection.as_str())
            .find_one(doc! { "id": id.to_lowercase() })
            .await
            .ok()
            .unwrap()
    }

    pub async fn update_last_block_indexed(
        &self,
        new_last_block_number: i32,
    ) {
        let mut connection = self.get_connection();

        diesel::update(
            sync_state::dsl::sync_state
                .filter(id.eq(DatabaseKeys::State.as_str())),
        )
        .set(last_block_indexed.eq(&new_last_block_number))
        .execute(&mut connection)
        .expect("unable to update sync state into database");
    }

    pub async fn update_factory(&self, data: &DatabaseFactory) {
        return self
            .update::<DatabaseFactory>(
                DatabaseKeys::Factory,
                DatabaseKeys::Factory.as_str(),
                data,
            )
            .await;
    }

    pub async fn update_token(&self, data: &DatabaseToken) {
        return self
            .update::<DatabaseToken>(DatabaseKeys::Tokens, &data.id, data)
            .await;
    }

    pub async fn update_pair(&self, data: &DatabasePair) {
        return self
            .update::<DatabasePair>(DatabaseKeys::Pairs, &data.id, data)
            .await;
    }

    pub async fn update_burn(&self, data: &DatabaseBurn) {
        return self
            .update::<DatabaseBurn>(DatabaseKeys::Burns, &data.id, data)
            .await;
    }

    pub async fn update_mint(&self, data: &DatabaseMint) {
        return self
            .update::<DatabaseMint>(DatabaseKeys::Mints, &data.id, data)
            .await;
    }

    pub async fn update_bundle(&self, data: &DatabaseBundle) {
        return self
            .update::<DatabaseBundle>(
                DatabaseKeys::Bundle,
                DatabaseKeys::Bundle.as_str(),
                data,
            )
            .await;
    }

    pub async fn update_swap(&self, data: &DatabaseSwap) {
        return self
            .update::<DatabaseSwap>(DatabaseKeys::Swaps, &data.id, data)
            .await;
    }

    pub async fn update_transaction(&self, data: &DatabaseTransaction) {
        return self
            .update::<DatabaseTransaction>(
                DatabaseKeys::Transactions,
                &data.hash,
                data,
            )
            .await;
    }

    pub async fn update_factory_day_data(
        &self,
        data: &DatabaseFactoryDayData,
    ) {
        return self
            .update::<DatabaseFactoryDayData>(
                DatabaseKeys::FactoryDayData,
                &data.id,
                data,
            )
            .await;
    }

    pub async fn update_pair_day_data(&self, data: &DatabasePairDayData) {
        return self
            .update::<DatabasePairDayData>(
                DatabaseKeys::PairDayData,
                &data.id,
                data,
            )
            .await;
    }

    pub async fn update_pair_hour_data(
        &self,
        data: &DatabasePairHourData,
    ) {
        return self
            .update::<DatabasePairHourData>(
                DatabaseKeys::PairHourData,
                &data.id,
                data,
            )
            .await;
    }

    pub async fn update_token_day_data(
        &self,
        data: &DatabaseTokenDayData,
    ) {
        return self
            .update::<DatabaseTokenDayData>(
                DatabaseKeys::TokenDayData,
                &data.id,
                data,
            )
            .await;
    }

    async fn update<T>(&self, collection: DatabaseKeys, id: &str, data: &T)
    where
        T: Serialize + Send + Sync + Unpin + 'static,
    {
        let filter = doc! { "id": id };

        let data_doc = to_document(data).unwrap();

        let doc = doc! { "$set": data_doc };

        self.db
            .collection::<T>(collection.as_str())
            .update_one(filter, doc)
            .upsert(true)
            .await
            .unwrap();
    }
}
