use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::db::Database;

sol! {
    event Transfer(address indexed,address indexed,uint256);
}

pub async fn handle_transfer(log: Log, db: &Database) {
    let event = Transfer::decode_log(&log.inner, true).unwrap();
}
/*
export function handleTransfer(event: Transfer): void {
    // ignore initial transfers for first adds
    if (event.params.to.toHexString() === ADDRESS_ZERO && event.params.value.equals(U256.fromI32(1000))) {
      return
    }

    const factory = UniswapFactory.load(FACTORY_ADDRESS)!
    const transactionHash = event.transaction.hash.toHexString()

    // user stats
    const from = event.params.from
    createUser(from)
    const to = event.params.to
    createUser(to)

    // get pair and load contract
    const pair = Pair.load(event.address.toHexString())!

    // liquidity token amount being transfered
    const value = convertTokenToDecimal(event.params.value, BI_18)

    // get or create transaction
    let transaction = Transaction.load(transactionHash)
    if (transaction === null) {
      transaction = new Transaction(transactionHash)
      transaction.blockNumber = event.block.number
      transaction.timestamp = event.block.timestamp
      transaction.mints = []
      transaction.burns = []
      transaction.swaps = []
    }

    // mints
    const mints = transaction.mints
    // part of the erc-20 standard (which is also the pool), whenever you mint new tokens, the from address is 0x0..0
    // the pool is also the erc-20 that gets minted and transferred around
    if (from.toHexString() === ADDRESS_ZERO) {
      // update total supply
      pair.totalSupply = pair.totalSupply.plus(value)
      pair.save()

      // create new mint if no mints so far or if last one is done already
      // transfers and mints come in pairs, but there could be a case where that doesn't happen and it might break
      // this is to make sure all the mints are under the same transaction
      if (mints.length === 0 || isCompleteMint(mints[mints.length - 1])) {
        const mint = new MintEvent(
          event.transaction.hash.toHexString().concat('-').concat(U256.fromI32(mints.length).toString())
        )
        mint.transaction = transaction.id
        mint.pair = pair.id
        mint.to = to
        mint.liquidity = value
        mint.timestamp = transaction.timestamp
        mint.transaction = transaction.id
        mint.save()

        // update mints in transaction
        transaction.mints = mints.concat([mint.id])

        // save entities
        transaction.save()
        factory.save()
      }
    }

    // case where direct send first on ETH withdrawls
    // for every burn event, there is a transfer first from the LP to the pool (erc-20)
    // when you LP, you get an ERC-20 token which is the accounting token of the LP position
    // the thing that's actually getting transfered is the LP account token
    if (event.params.to.toHexString() === pair.id) {
      const burns = transaction.burns
      const burn = new BurnEvent(
        event.transaction.hash.toHexString().concat('-').concat(U256.fromI32(burns.length).toString())
      )
      burn.transaction = transaction.id
      burn.pair = pair.id
      burn.liquidity = value
      burn.timestamp = transaction.timestamp
      burn.to = event.params.to
      burn.sender = event.params.from
      burn.needsComplete = true
      burn.transaction = transaction.id
      burn.save()

      // TODO: Consider using .concat() for handling array updates to protect
      // against unintended side effects for other code paths.
      burns.push(burn.id)
      transaction.burns = burns
      transaction.save()
    }

    // burn
    // there's two transfers for the LP token,
    // first its going to move from the LP back to the pool, and then it will go from the pool to the zero address
    if (event.params.to.toHexString() === ADDRESS_ZERO && event.params.from.toHexString() === pair.id) {
      pair.totalSupply = pair.totalSupply.minus(value)
      pair.save()

      // this is a new instance of a logical burn
      const burns = transaction.burns
      let burn: BurnEvent
      // this block creates the burn or gets the reference to it if it already exists
      if (burns.length > 0) {
        const currentBurn = BurnEvent.load(burns[burns.length - 1])!
        if (currentBurn.needsComplete) {
          burn = currentBurn as BurnEvent
        } else {
          burn = new BurnEvent(
            event.transaction.hash.toHexString().concat('-').concat(U256.fromI32(burns.length).toString())
          )
          burn.transaction = transaction.id
          burn.needsComplete = false
          burn.pair = pair.id
          burn.liquidity = value
          burn.transaction = transaction.id
          burn.timestamp = transaction.timestamp
        }
      } else {
        burn = new BurnEvent(
          event.transaction.hash.toHexString().concat('-').concat(U256.fromI32(burns.length).toString())
        )
        burn.transaction = transaction.id
        burn.needsComplete = false
        burn.pair = pair.id
        burn.liquidity = value
        burn.transaction = transaction.id
        burn.timestamp = transaction.timestamp
      }

      // if this logical burn included a fee mint, account for this
      // what is a fee mint?
      // how are fees collected on v2?
      // when you're an LP in v2, you're earning fees in terms of LP tokens, so when you go to burn your position, burn and collect fees at the same time
      // protocol is sending the LP something and we think it's a mint when it's not and it's really fees
      if (mints.length !== 0 && !isCompleteMint(mints[mints.length - 1])) {
        const mint = MintEvent.load(mints[mints.length - 1])!
        burn.feeTo = mint.to
        burn.feeLiquidity = mint.liquidity
        // remove the logical mint
        store.remove('Mint', mints[mints.length - 1])
        // update the transaction

        // TODO: Consider using .slice().pop() to protect against unintended
        // side effects for other code paths.
        mints.pop()
        transaction.mints = mints
        transaction.save()
      }
      // when you collect fees or burn liquidity what are the events that get triggered
      // not sure why this replaced the last one instead of updating
      burn.save()
      // if accessing last one, replace it
      if (burn.needsComplete) {
        // TODO: Consider using .slice(0, -1).concat() to protect against
        // unintended side effects for other code paths.
        burns[burns.length - 1] = burn.id
      }
      // else add new one
      else {
        // TODO: Consider using .concat() for handling array updates to protect
        // against unintended side effects for other code paths.
        burns.push(burn.id)
      }
      transaction.burns = burns
      transaction.save()
    }

    transaction.save()
  }
   */
