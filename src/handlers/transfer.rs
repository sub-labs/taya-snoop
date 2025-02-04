use alloy::{
    primitives::Address, rpc::types::Log, sol, sol_types::SolEvent,
};
use fastnum::udec256;

use crate::{
    db::{
        models::{
            burn::{BurnData, DatabaseBurn},
            mint::DatabaseMint,
            transaction::DatabaseTransaction,
        },
        Database,
    },
    utils::format::parse_ud256,
};

use super::utils::convert_token_to_decimal;

sol! {
    event Transfer(address indexed from,address indexed to,uint256 value);
}

async fn is_complete_mint(mint_id: String, db: &Database) -> bool {
    let mint = db.get_mint(&mint_id).await.unwrap();

    mint.sender == Address::ZERO.to_string()
}

pub async fn handle_transfer(
    log: Log,
    block_timestamp: i64,
    db: &Database,
) {
    let event = Transfer::decode_log(&log.inner, true).unwrap();

    if event.from == Address::ZERO
        && parse_ud256(event.value).eq(&udec256!(1000))
    {
        return;
    }

    let factory = db.get_factory().await;
    let transaction_hash = log.transaction_hash.unwrap().to_string();

    let from = event.from.to_string().to_lowercase();
    let to = event.to.to_string().to_lowercase();

    let mut pair = db.get_pair(&event.address.to_string()).await.unwrap();

    let value =
        convert_token_to_decimal(&parse_ud256(event.value), &udec256!(18));

    let block_number = log.block_number.unwrap() as i64;

    let mut transaction = match db.get_transaction(&transaction_hash).await
    {
        Some(transaction) => transaction,
        None => DatabaseTransaction::new(
            transaction_hash.clone(),
            block_number,
            block_timestamp,
        ),
    };

    if from == Address::ZERO.to_string() {
        pair.total_supply = pair.total_supply.add(value);

        db.update_pair(&pair).await;

        if transaction.mints.is_empty()
            || is_complete_mint(
                transaction.mints.last().unwrap().to_string(),
                db,
            )
            .await
        {
            let mint_id = format!(
                "{}-{}",
                transaction_hash.clone(),
                transaction.mints.len()
            );

            let mint = DatabaseMint::new(
                mint_id,
                transaction_hash.clone(),
                block_timestamp,
                pair.id.clone(),
                to.clone(),
                log.log_index.unwrap() as i64,
            );

            db.update_mint(&mint).await;

            transaction.mints.push(mint.id);

            db.update_transaction(&transaction).await;
            db.update_factory(&factory).await;
        }
    }

    if to == pair.id {
        let burn_id = format!(
            "{}-{}",
            transaction_hash.clone(),
            transaction.burns.len()
        );

        let burn = DatabaseBurn::new(
            burn_id,
            transaction_hash.clone(),
            block_timestamp,
            log.log_index.unwrap() as i64,
            BurnData {
                sender: Some(from.to_string()),
                liquidity: value,
                pair: pair.id.clone(),
                to: Some(to.clone()),
                needs_complete: true,
            },
        );

        db.update_burn(&burn).await;

        transaction.burns.push(burn.id.clone());

        db.update_transaction(&transaction).await;
    }

    if to == Address::ZERO.to_string() && from == pair.id {
        pair.total_supply = pair.total_supply.min(value);

        db.update_pair(&pair).await;

        let mut burn: DatabaseBurn;

        if !transaction.burns.is_empty() {
            let current_burn = db
                .get_burn(transaction.burns.last().unwrap())
                .await
                .unwrap();

            if current_burn.needs_complete {
                burn = current_burn
            } else {
                let burn_id = format!(
                    "{}-{}",
                    transaction_hash.clone(),
                    transaction.burns.len()
                );

                burn = DatabaseBurn::new(
                    burn_id,
                    transaction_hash.clone(),
                    block_timestamp,
                    log.log_index.unwrap() as i64,
                    BurnData {
                        sender: None,
                        liquidity: value,
                        pair: pair.id.clone(),
                        to: None,
                        needs_complete: false,
                    },
                )
            }
        } else {
            let burn_id = format!(
                "{}-{}",
                transaction_hash.clone(),
                transaction.burns.len()
            );

            burn = DatabaseBurn::new(
                burn_id,
                transaction_hash.clone(),
                block_timestamp,
                log.log_index.unwrap() as i64,
                BurnData {
                    sender: None,
                    liquidity: value,
                    pair: pair.id.clone(),
                    to: None,
                    needs_complete: false,
                },
            )
        }

        if !transaction.mints.is_empty()
            && !is_complete_mint(
                transaction.mints.last().unwrap().to_string(),
                db,
            )
            .await
        {
            let mint = db
                .get_mint(transaction.mints.last().unwrap())
                .await
                .unwrap();

            burn.fee_to = mint.to;
            burn.fee_liquidity = mint.liquidity;

            // store.remove('Mint', mints[mints.length - 1]) Skip Mint removal

            transaction.mints.pop().unwrap();
            db.update_transaction(&transaction).await;
        }

        db.update_burn(&burn).await;

        if burn.needs_complete {
            transaction
                .burns
                .insert(transaction.burns.len() - 1, burn.id.clone());
        } else {
            transaction.burns.push(burn.id);
        }

        db.update_transaction(&transaction).await;
    }

    db.update_transaction(&transaction).await;
}
