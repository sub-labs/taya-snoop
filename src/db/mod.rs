pub mod models;

use crate::{chains::Chain, configs::Config};

use log::*;
use models::{factory::DatabaseFactory, sync_state::DatabaseSyncState};
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use serde::Serialize;

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db: mongodb::Database,
}

pub enum DatabaseKeys {
    State,
    Factory,
    Logs,
    Burns,
    Mints,
    Swaps,
    Transactions,
    Users,
    Bundle,
    Tokens,
    Pairs,
    DexDayData,
    PairHourData,
    PairDayData,
    TokenDayData,
}

impl DatabaseKeys {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseKeys::State => "sync_state",
            DatabaseKeys::Factory => "taya_swap",
            DatabaseKeys::Logs => "logs",
            DatabaseKeys::Burns => "burns",
            DatabaseKeys::Mints => "mints",
            DatabaseKeys::Swaps => "swaps",
            DatabaseKeys::Transactions => "transactions",
            DatabaseKeys::Users => "users",
            DatabaseKeys::Bundle => "bundle",
            DatabaseKeys::Tokens => "tokens",
            DatabaseKeys::Pairs => "pairs",
            DatabaseKeys::DexDayData => "dex_day_data",
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

    pub async fn get_factory(&self, config: &Config) -> DatabaseFactory {
        let factory_key = DatabaseKeys::Factory.as_str();

        let factory = self
            .db
            .collection::<DatabaseFactory>(factory_key)
            .find_one(doc! { "id": { "$eq": factory_key}})
            .await
            .unwrap();

        match factory {
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

    pub async fn update_factory(&self, factory: &DatabaseFactory) {
        let factory_key = DatabaseKeys::Factory.as_str();

        let filter = doc! { "id": factory_key };
        let update = doc! {
        "$set": {
            "pair_count": factory.pair_count,
            "total_volume_usd": factory.total_volume_eth,
            "total_volume_eth": factory.total_volume_eth,
            "untracked_volume_usd": factory.untracked_volume_usd,
            "total_liquidity_usd": factory.total_liquidity_usd,
            "total_liquidity_eth": factory.total_liquidity_eth,
            "tx_count": factory.tx_count,
            "pairs": factory.pairs.clone()
        }};

        self.db
            .collection::<DatabaseFactory>(factory_key)
            .update_one(filter, update)
            .await
            .unwrap();
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
            .await
            .unwrap();
    }

    pub async fn store<T>(&self, key: DatabaseKeys, data: &Vec<T>)
    where
        T: Serialize + Send + Sync + Unpin + 'static,
    {
        if !data.is_empty() {
            self.db
                .collection::<T>(key.as_str())
                .insert_many(data)
                .await
                .unwrap();
        }
    }
}
