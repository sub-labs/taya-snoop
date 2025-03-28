pub mod models;
pub mod schema;

use crate::chains::Chain;

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
use mongodb::{
    bson::{doc, to_document},
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db: mongodb::Database,
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

        let mut client_options =
            ClientOptions::parse(db_url).await.unwrap();

        let server_api =
            ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let db = Client::with_options(client_options)
            .unwrap()
            .database(DATABASE);

        Self { chain, db }
    }

    pub async fn get_last_block_indexed(&self) -> i64 {
        let sync_state_key = DatabaseKeys::State.as_str();

        let sync_state = self
            .db
            .collection::<DatabaseSyncState>(sync_state_key)
            .find_one(doc! { "id": { "$eq": sync_state_key}})
            .await
            .unwrap();

        match sync_state {
            Some(sync_state) => sync_state.last_block_indexed,
            None => {
                let new_sync_state = DatabaseSyncState {
                    id: sync_state_key.to_owned(),
                    last_block_indexed: 0,
                };

                self.db
                    .collection::<DatabaseSyncState>(sync_state_key)
                    .insert_one(new_sync_state)
                    .await
                    .unwrap();

                0
            }
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
        new_last_block_number: i64,
    ) {
        let sync_state_key = DatabaseKeys::State.as_str();

        let filter = doc! { "id": sync_state_key };
        let update = doc! { "$set": doc! {"last_block_indexed": new_last_block_number} };

        self.db
            .collection::<DatabaseSyncState>(sync_state_key)
            .update_one(filter, update)
            .upsert(true)
            .await
            .unwrap();
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
