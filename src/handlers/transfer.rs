use alloy::{rpc::types::Log, sol, sol_types::SolEvent};
use bigdecimal::BigDecimal;

use crate::{
    db::{
        models::{
            burn::DatabaseBurn, mint::DatabaseMint,
            transaction::DatabaseTransaction,
        },
        Database,
    },
    utils::format::{address_zero, convert_token_to_decimal, parse_u256},
};

sol! {
    event Transfer(address indexed from,address indexed to,uint256 value);
}

async fn is_complete_mint(mint_id: String, db: &Database) -> bool {
    let mint = db.get_mint(&mint_id).await.unwrap();

    mint.sender == address_zero()
}

pub async fn handle_transfer(
    log: Log,
    block_timestamp: i32,
    db: &Database,
) {
    let event = Transfer::decode_log(&log.inner, true).unwrap();
    let from_address = event.from.to_string().to_lowercase();
    let to_address = event.to.to_string().to_lowercase();

    if from_address == address_zero()
        && parse_u256(event.value) == BigDecimal::from(1000)
    {
        return;
    }

    let transaction_hash = log.transaction_hash.unwrap().to_string();
    let pair_address = event.address.to_string().to_lowercase();
    let block_number = log.block_number.unwrap() as i32;
    let value = convert_token_to_decimal(&parse_u256(event.value), 18);
    let log_index = log.log_index.unwrap() as i32;

    let (factory, pair_result, transaction_result) = tokio::join!(
        db.get_factory(),
        db.get_pair(&pair_address),
        db.get_transaction(&transaction_hash)
    );

    let mut pair = pair_result.unwrap();

    let mut transaction = transaction_result.unwrap_or_else(|| {
        DatabaseTransaction::new(
            transaction_hash.clone(),
            block_number,
            block_timestamp,
        )
    });

    if from_address == address_zero() {
        pair.total_supply += value.clone();

        if transaction.mints.is_empty()
            || is_complete_mint(
                transaction.mints.last().unwrap().clone().unwrap(),
                db,
            )
            .await
        {
            let mint_id = format!(
                "{}-{}",
                transaction_hash,
                transaction.mints.len()
            );
            let mint = DatabaseMint::new(
                mint_id,
                transaction_hash.clone(),
                block_timestamp,
                pair_address.clone(),
                to_address.clone(),
                log_index,
            );
            transaction.mints.push(Some(mint.id.clone()));
            tokio::join!(
                db.update_mint(&mint),
                db.update_transaction(&transaction),
                db.update_factory(&factory)
            );
        }
    }

    if to_address == pair_address {
        let burn_id =
            format!("{}-{}", transaction_hash, transaction.burns.len());

        let burn = DatabaseBurn::new(
            burn_id,
            transaction_hash.clone(),
            block_timestamp,
            log_index,
            pair_address.clone(),
            to_address.clone(),
            value.clone(),
            from_address.clone(),
            true,
        );

        transaction.burns.push(Some(burn.id.clone()));
        tokio::join!(
            db.update_burn(&burn),
            db.update_transaction(&transaction)
        );
    }

    if to_address == address_zero() && from_address == pair_address {
        pair.total_supply -= value.clone();

        let burn = if !transaction.burns.is_empty() {
            let last_burn_id =
                transaction.burns.last().unwrap().as_ref().unwrap();

            let current_burn = db.get_burn(last_burn_id).await.unwrap();

            if current_burn.needs_complete {
                current_burn
            } else {
                let burn_id = format!(
                    "{}-{}",
                    transaction_hash,
                    transaction.burns.len()
                );
                DatabaseBurn::new(
                    burn_id,
                    transaction_hash.clone(),
                    block_timestamp,
                    log_index,
                    pair_address.clone(),
                    to_address.clone(),
                    value.clone(),
                    address_zero(),
                    false,
                )
            }
        } else {
            let burn_id = format!(
                "{}-{}",
                transaction_hash,
                transaction.burns.len()
            );
            DatabaseBurn::new(
                burn_id,
                transaction_hash.clone(),
                block_timestamp,
                log_index,
                pair_address.clone(),
                address_zero(),
                value.clone(),
                address_zero(),
                false,
            )
        };

        let mut burn = burn;

        if !transaction.mints.is_empty() {
            let last_mint_id =
                transaction.mints.last().unwrap().clone().unwrap();

            if !is_complete_mint(last_mint_id.clone(), db).await {
                let mint_id =
                    transaction.mints.last().unwrap().as_ref().unwrap();
                let mint = db.get_mint(mint_id).await.unwrap();
                burn.fee_to = mint.to;
                burn.fee_liquidity = mint.liquidity;
                transaction.mints.pop();
                db.update_transaction(&transaction).await;
            }
        }

        db.update_burn(&burn).await;

        if burn.needs_complete {
            transaction.burns.insert(
                transaction.burns.len() - 1,
                Some(burn.id.clone()),
            );
        } else {
            transaction.burns.push(Some(burn.id));
        }

        db.update_transaction(&transaction).await;
    }

    tokio::join!(
        db.update_pair(&pair),
        db.update_transaction(&transaction)
    );
}
