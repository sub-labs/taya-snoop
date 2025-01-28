pub mod models;

use crate::chains::Chain;

use log::*;
use models::sync_state::DatabaseSyncState;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};

#[derive(Clone)]
pub struct Database {
    pub chain: Chain,
    pub db: mongodb::Database,
}

pub enum DatabaseKeys {
    State,
}

impl DatabaseKeys {
    pub fn as_str(&self) -> &'static str {
        match self {
            DatabaseKeys::State => "sync_state",
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
}
