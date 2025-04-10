pub mod models;
pub mod schema;

use std::collections::HashMap;

use crate::chains::Chain;

use diesel::{
    upsert::excluded, BoolExpressionMethods, Connection,
    ExpressionMethods, OptionalExtension, PgConnection, QueryDsl,
    QueryResult, RunQueryDsl,
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
    pub pairs_day_data: HashMap<String, DatabasePairDayData>,
    pub pairs_hour_data: HashMap<String, DatabasePairHourData>,
    pub tokens_day_data: HashMap<String, DatabaseTokenDayData>,
    pub dex_day_data: HashMap<String, DatabaseDexDayData>,
}

impl StorageCache {
    pub async fn store(&self) {
        let pairs: Vec<DatabasePair> =
            self.pairs.clone().into_values().collect();

        let tokens: Vec<DatabaseToken> =
            self.tokens.clone().into_values().collect();

        let transactions: Vec<DatabaseTransaction> =
            self.transactions.clone().into_values().collect();

        let mints: Vec<DatabaseMint> =
            self.mints.clone().into_values().collect();

        let swaps: Vec<DatabaseSwap> =
            self.swaps.clone().into_values().collect();

        let burns: Vec<DatabaseBurn> =
            self.burns.clone().into_values().collect();

        let pairs_day_data: Vec<DatabasePairDayData> =
            self.pairs_day_data.clone().into_values().collect();

        let pairs_hour_data: Vec<DatabasePairHourData> =
            self.pairs_hour_data.clone().into_values().collect();

        let tokens_day_data: Vec<DatabaseTokenDayData> =
            self.tokens_day_data.clone().into_values().collect();

        let dex_day_data: Vec<DatabaseDexDayData> =
            self.dex_day_data.clone().into_values().collect();

        tokio::join!(
            self.db.update_factory(&self.factory),
            self.db.update_bundle(&self.bundle),
            self.db.update_pairs(&pairs),
            self.db.update_tokens(&tokens),
            self.db.update_burns(&burns),
            self.db.update_mints(&mints),
            self.db.update_swaps(&swaps),
            self.db.update_transactions(&transactions),
            self.db.update_dexes_day_data(&dex_day_data),
            self.db.update_pairs_day_data(&pairs_day_data),
            self.db.update_pairs_hour_data(&pairs_hour_data),
            self.db.update_tokens_day_data(&tokens_day_data)
        );
    }
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

    pub async fn update_tokens(&self, data: &Vec<DatabaseToken>) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(tokens::dsl::tokens)
            .values(data)
            .on_conflict(tokens::id)
            .do_update()
            .set((
                tokens::id.eq(excluded(tokens::id)),
                tokens::symbol.eq(excluded(tokens::symbol)),
                tokens::name.eq(excluded(tokens::name)),
                tokens::decimals.eq(excluded(tokens::decimals)),
                tokens::total_supply.eq(excluded(tokens::total_supply)),
                tokens::trade_volume.eq(excluded(tokens::trade_volume)),
                tokens::trade_volume_usd
                    .eq(excluded(tokens::trade_volume_usd)),
                tokens::untracked_volume_usd
                    .eq(excluded(tokens::untracked_volume_usd)),
                tokens::tx_count.eq(excluded(tokens::tx_count)),
                tokens::total_liquidity
                    .eq(excluded(tokens::total_liquidity)),
                tokens::derived_eth.eq(excluded(tokens::derived_eth)),
            ))
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

    pub async fn update_pairs(&self, data: &Vec<DatabasePair>) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pairs::dsl::pairs)
            .values(data)
            .on_conflict(pairs::id)
            .do_update()
            .set((
                pairs::token0.eq(excluded(pairs::token0)),
                pairs::token1.eq(excluded(pairs::token1)),
                pairs::reserve0.eq(excluded(pairs::reserve0)),
                pairs::reserve1.eq(excluded(pairs::reserve1)),
                pairs::total_supply.eq(excluded(pairs::total_supply)),
                pairs::reserve_eth.eq(excluded(pairs::reserve_eth)),
                pairs::reserve_usd.eq(excluded(pairs::reserve_usd)),
                pairs::tracked_reserve_eth
                    .eq(excluded(pairs::tracked_reserve_eth)),
                pairs::token0_price.eq(excluded(pairs::token0_price)),
                pairs::token1_price.eq(excluded(pairs::token1_price)),
                pairs::volume_token0.eq(excluded(pairs::volume_token0)),
                pairs::volume_token1.eq(excluded(pairs::volume_token1)),
                pairs::volume_usd.eq(excluded(pairs::volume_usd)),
                pairs::untracked_volume_usd
                    .eq(excluded(pairs::untracked_volume_usd)),
                pairs::tx_count.eq(excluded(pairs::tx_count)),
                pairs::created_at_timestamp
                    .eq(excluded(pairs::created_at_timestamp)),
                pairs::created_at_block_number
                    .eq(excluded(pairs::created_at_block_number)),
                pairs::liquidity_provider_count
                    .eq(excluded(pairs::liquidity_provider_count)),
            ))
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

    pub async fn update_burns(&self, data: &Vec<DatabaseBurn>) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(burns::dsl::burns)
            .values(data)
            .on_conflict(burns::id)
            .do_update()
            .set((
                burns::id.eq(excluded(burns::id)),
                burns::transaction.eq(excluded(burns::transaction)),
                burns::timestamp.eq(excluded(burns::timestamp)),
                burns::pair.eq(excluded(burns::pair)),
                burns::liquidity.eq(excluded(burns::liquidity)),
                burns::sender.eq(excluded(burns::sender)),
                burns::amount0.eq(excluded(burns::amount0)),
                burns::amount1.eq(excluded(burns::amount1)),
                burns::to.eq(excluded(burns::to)),
                burns::log_index.eq(excluded(burns::log_index)),
                burns::amount_usd.eq(excluded(burns::amount_usd)),
                burns::id.eq(excluded(burns::id)),
                burns::needs_complete.eq(excluded(burns::needs_complete)),
                burns::fee_to.eq(excluded(burns::fee_to)),
                burns::fee_liquidity.eq(excluded(burns::fee_liquidity)),
            ))
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

    pub async fn update_mints(&self, data: &Vec<DatabaseMint>) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(mints::dsl::mints)
            .values(data)
            .on_conflict(mints::id)
            .do_update()
            .set((
                mints::id.eq(excluded(mints::id)),
                mints::transaction.eq(excluded(mints::transaction)),
                mints::timestamp.eq(excluded(mints::timestamp)),
                mints::pair.eq(excluded(mints::pair)),
                mints::to.eq(excluded(mints::to)),
                mints::liquidity.eq(excluded(mints::liquidity)),
                mints::sender.eq(excluded(mints::sender)),
                mints::amount0.eq(excluded(mints::amount0)),
                mints::amount1.eq(excluded(mints::amount1)),
                mints::log_index.eq(excluded(mints::log_index)),
                mints::amount_usd.eq(excluded(mints::amount_usd)),
                mints::fee_to.eq(excluded(mints::fee_to)),
                mints::fee_liquidity.eq(excluded(mints::fee_liquidity)),
            ))
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

    pub async fn update_swaps(&self, data: &Vec<DatabaseSwap>) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(swaps::dsl::swaps)
            .values(data)
            .on_conflict(swaps::id)
            .do_update()
            .set((
                swaps::id.eq(excluded(swaps::id)),
                swaps::transaction.eq(excluded(swaps::transaction)),
                swaps::timestamp.eq(excluded(swaps::timestamp)),
                swaps::pair.eq(excluded(swaps::pair)),
                swaps::sender.eq(excluded(swaps::sender)),
                swaps::from.eq(excluded(swaps::from)),
                swaps::amount0_in.eq(excluded(swaps::amount0_in)),
                swaps::amount1_in.eq(excluded(swaps::amount1_in)),
                swaps::amount0_out.eq(excluded(swaps::amount0_out)),
                swaps::amount1_out.eq(excluded(swaps::amount1_out)),
                swaps::to.eq(excluded(swaps::to)),
                swaps::log_index.eq(excluded(swaps::log_index)),
                swaps::amount_usd.eq(excluded(swaps::amount_usd)),
            ))
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

    pub async fn update_transactions(
        &self,
        data: &Vec<DatabaseTransaction>,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(transactions::dsl::transactions)
            .values(data)
            .on_conflict(transactions::id)
            .do_update()
            .set((
                transactions::id.eq(excluded(transactions::id)),
                transactions::block_number
                    .eq(excluded(transactions::block_number)),
                transactions::timestamp
                    .eq(excluded(transactions::timestamp)),
                transactions::mints.eq(excluded(transactions::mints)),
                transactions::swaps.eq(excluded(transactions::swaps)),
                transactions::burns.eq(excluded(transactions::burns)),
            ))
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

    pub async fn update_dexes_day_data(
        &self,
        data: &Vec<DatabaseDexDayData>,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(dex_day_data::dsl::dex_day_data)
            .values(data)
            .on_conflict(dex_day_data::id)
            .do_update()
            .set((
                dex_day_data::id.eq(excluded(dex_day_data::id)),
                dex_day_data::date.eq(excluded(dex_day_data::date)),
                dex_day_data::daily_volume_eth
                    .eq(excluded(dex_day_data::daily_volume_eth)),
                dex_day_data::daily_volume_usd
                    .eq(excluded(dex_day_data::daily_volume_usd)),
                dex_day_data::daily_volume_untracked
                    .eq(excluded(dex_day_data::daily_volume_untracked)),
                dex_day_data::total_volume_eth
                    .eq(excluded(dex_day_data::total_volume_eth)),
                dex_day_data::total_liquidity_eth
                    .eq(excluded(dex_day_data::total_liquidity_eth)),
                dex_day_data::total_volume_usd
                    .eq(excluded(dex_day_data::total_volume_usd)),
                dex_day_data::total_liquidity_usd
                    .eq(excluded(dex_day_data::total_liquidity_usd)),
                dex_day_data::tx_count
                    .eq(excluded(dex_day_data::tx_count)),
            ))
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

    pub async fn update_pairs_day_data(
        &self,
        data: &Vec<DatabasePairDayData>,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pair_day_data::dsl::pair_day_data)
            .values(data)
            .on_conflict(pair_day_data::id)
            .do_update()
            .set((
                pair_day_data::id.eq(excluded(pair_day_data::id)),
                pair_day_data::date.eq(excluded(pair_day_data::date)),
                pair_day_data::pair_address
                    .eq(excluded(pair_day_data::pair_address)),
                pair_day_data::token0.eq(excluded(pair_day_data::token0)),
                pair_day_data::token1.eq(excluded(pair_day_data::token1)),
                pair_day_data::reserve0
                    .eq(excluded(pair_day_data::reserve0)),
                pair_day_data::reserve1
                    .eq(excluded(pair_day_data::reserve1)),
                pair_day_data::total_supply
                    .eq(excluded(pair_day_data::total_supply)),
                pair_day_data::reserve_usd
                    .eq(excluded(pair_day_data::reserve_usd)),
                pair_day_data::daily_volume_token0
                    .eq(excluded(pair_day_data::daily_volume_token0)),
                pair_day_data::daily_volume_token1
                    .eq(excluded(pair_day_data::daily_volume_token1)),
                pair_day_data::daily_volume_usd
                    .eq(excluded(pair_day_data::daily_volume_usd)),
                pair_day_data::daily_txns
                    .eq(excluded(pair_day_data::daily_txns)),
            ))
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

    pub async fn update_pairs_hour_data(
        &self,
        data: &Vec<DatabasePairHourData>,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(pair_hour_data::dsl::pair_hour_data)
            .values(data)
            .on_conflict(pair_hour_data::id)
            .do_update()
            .set((
                pair_hour_data::id.eq(excluded(pair_hour_data::id)),
                pair_hour_data::hour_start_unix
                    .eq(excluded(pair_hour_data::hour_start_unix)),
                pair_hour_data::pair.eq(excluded(pair_hour_data::pair)),
                pair_hour_data::reserve0
                    .eq(excluded(pair_hour_data::reserve0)),
                pair_hour_data::reserve1
                    .eq(excluded(pair_hour_data::reserve1)),
                pair_hour_data::total_supply
                    .eq(excluded(pair_hour_data::total_supply)),
                pair_hour_data::reserve_usd
                    .eq(excluded(pair_hour_data::reserve_usd)),
                pair_hour_data::hourly_volume_token0
                    .eq(excluded(pair_hour_data::hourly_volume_token0)),
                pair_hour_data::hourly_volume_token1
                    .eq(excluded(pair_hour_data::hourly_volume_token1)),
                pair_hour_data::hourly_volume_usd
                    .eq(excluded(pair_hour_data::hourly_volume_usd)),
                pair_hour_data::hourly_txns
                    .eq(excluded(pair_hour_data::hourly_txns)),
            ))
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

    pub async fn update_tokens_day_data(
        &self,
        data: &Vec<DatabaseTokenDayData>,
    ) {
        let mut connection: PgConnection = self.get_connection();

        diesel::insert_into(token_day_data::dsl::token_day_data)
            .values(data)
            .on_conflict(token_day_data::id)
            .do_update()
            .set((
                token_day_data::id.eq(excluded(token_day_data::id)),
                token_day_data::date.eq(excluded(token_day_data::date)),
                token_day_data::token.eq(excluded(token_day_data::token)),
                token_day_data::daily_volume_token
                    .eq(excluded(token_day_data::daily_volume_token)),
                token_day_data::daily_volume_eth
                    .eq(excluded(token_day_data::daily_volume_eth)),
                token_day_data::daily_volume_usd
                    .eq(excluded(token_day_data::daily_volume_usd)),
                token_day_data::daily_txns
                    .eq(excluded(token_day_data::daily_txns)),
                token_day_data::total_liquidity_token
                    .eq(excluded(token_day_data::total_liquidity_token)),
                token_day_data::total_liquidity_eth
                    .eq(excluded(token_day_data::total_liquidity_eth)),
                token_day_data::total_liquidity_usd
                    .eq(excluded(token_day_data::total_liquidity_usd)),
                token_day_data::price_usd
                    .eq(excluded(token_day_data::price_usd)),
            ))
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
