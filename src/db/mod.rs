pub mod models;

use crate::chains::Chain;

use log::*;
use models::{
    bundle::DatabaseBundle, factory::DatabaseFactory, pair::DatabasePair,
    sync_state::DatabaseSyncState, token::DatabaseToken,
};
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
    Bundle,
    Logs,
    Burns,
    Mints,
    Swaps,
    Transactions,
    Users,
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
            DatabaseKeys::Bundle => "bundle",
            DatabaseKeys::Logs => "logs",
            DatabaseKeys::Burns => "burns",
            DatabaseKeys::Mints => "mints",
            DatabaseKeys::Swaps => "swaps",
            DatabaseKeys::Transactions => "transactions",
            DatabaseKeys::Users => "users",
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

    pub async fn get_token(
        &self,
        token_id: String,
    ) -> Option<DatabaseToken> {
        self.db
            .collection::<DatabaseToken>(DatabaseKeys::Tokens.as_str())
            .find_one(doc! { "id": { "$eq": token_id}})
            .await
            .unwrap()
    }

    pub async fn get_pair(&self, pair_id: String) -> Option<DatabasePair> {
        self.db
            .collection::<DatabasePair>(DatabaseKeys::Pairs.as_str())
            .find_one(doc! { "id": { "$eq": pair_id}})
            .await
            .unwrap()
    }

    pub async fn get_bundle(&self) -> DatabaseBundle {
        let bundle_key = DatabaseKeys::Factory.as_str();

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

    pub async fn get_pairs(&self) -> Vec<String> {
        let factory = self.get_factory().await;

        factory.pairs
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
            .upsert(true)
            .await
            .unwrap();
    }

    pub async fn update_token(&self, token: &DatabaseToken) {
        let token_id = token.id.clone();

        let filter = doc! { "id": token_id };
        let update = doc! {
        "$set": {
            "symbol": token.symbol.clone(),
            "name": token.name.clone(),
            "decimals": token.decimals,
            "total_supply": token.total_supply.clone(),
            "trade_volume": token.trade_volume,
            "trade_volume_usd": token.trade_volume_usd,
            "untracked_volume_usd": token.untracked_volume_usd,
            "tx_count": token.tx_count,
            "total_liquidity": token.total_liquidity,
            "derived_eth": token.derived_eth,
        }};

        self.db
            .collection::<DatabaseToken>(DatabaseKeys::Tokens.as_str())
            .update_one(filter, update)
            .upsert(true)
            .await
            .unwrap();
    }

    pub async fn update_bundle(&self, bundle: &DatabaseBundle) {
        let bundle_key = DatabaseKeys::Bundle.as_str();

        let filter = doc! { "id": bundle_key };
        let update = doc! {
        "$set": {
            "eth_price": bundle.eth_price,
        }};

        self.db
            .collection::<DatabaseBundle>(bundle_key)
            .update_one(filter, update)
            .upsert(true)
            .await
            .unwrap();
    }

    pub async fn update_pair(&self, pair: &DatabasePair) {
        let pair_key = &pair.id;

        let filter = doc! { "id": pair_key };
        let update = doc! {
        "$set": {
            "token0": pair.token0.clone(),
            "token1": pair.token1.clone(),
            "reserve0": pair.reserve0,
            "reserve1": pair.reserve1,
            "total_supply": pair.total_supply,
            "reserve_eth": pair.reserve_eth,
            "reserve_usd": pair.reserve_usd,
            "tracked_reserve_eth": pair.tracked_reserve_eth,
            "token0_price": pair.token0_price,
            "token1_price": pair.token1_price,
            "volume_token0": pair.volume_token0,
            "volume_token1": pair.volume_token1,
            "volume_usd": pair.volume_usd,
            "untracked_volume_usd": pair.untracked_volume_usd,
            "tx_count": pair.tx_count,
            "created_at_timestamp": pair.created_at_timestamp,
            "created_at_block_number": pair.created_at_block_number,
            "liquidity_provider_count": pair.liquidity_provider_count,
        }};

        self.db
            .collection::<DatabasePair>(pair_key)
            .update_one(filter, update)
            .upsert(true)
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
            .upsert(true)
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
