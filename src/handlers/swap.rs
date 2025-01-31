use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::{db::Database, utils::format::parse_uint256};

use super::utils::convert_token_to_decimal;

sol! {
    event Swap(
        address indexed sender,
        uint amount0In,
        uint amount1In,
        uint amount0Out,
        uint amount1Out,
        address indexed to
    );
}

pub async fn handle_swap(log: Log, db: &Database) {
    let event = Swap::decode_log(&log.inner, true).unwrap();

    let pair = db.get_pair(event.address.to_string()).await.unwrap();

    let token0 = db.get_token(pair.token0).await;
    let token1 = db.get_token(pair.token1).await;

    if token0.is_none() || token1.is_none() {
        return;
    }

    let token0 = token0.unwrap();
    let token1 = token1.unwrap();

    let amount0_in = convert_token_to_decimal(
        &parse_uint256(event.amount0In),
        &token0.decimals,
    );

    let amount1_in = convert_token_to_decimal(
        &parse_uint256(event.amount1In),
        &token1.decimals,
    );

    let amount0_out = convert_token_to_decimal(
        &parse_uint256(event.amount0Out),
        &token0.decimals,
    );

    let amount1_out = convert_token_to_decimal(
        &parse_uint256(event.amount1Out),
        &token1.decimals,
    );

    let amount0_total = amount0_out.add(amount0_in);
    let amount1_total = amount1_out.add(amount1_in);
}

/*
export function handleSwap(event: Swap): void {


    // ETH/USD prices
    const bundle = Bundle.load('1')!

    // get total amounts of derived USD and ETH for tracking
    const derivedAmountETH = token1.derivedETH
      .times(amount1Total)
      .plus(token0.derivedETH.times(amount0Total))
      .div(UD256.fromString('2'))
    const derivedAmountUSD = derivedAmountETH.times(bundle.ethPrice)

    // only accounts for volume through white listed tokens
    const trackedAmountUSD = getTrackedVolumeUSD(
      amount0Total,
      token0 as Token,
      amount1Total,
      token1 as Token,
      pair as Pair
    )

    let trackedAmountETH: UD256
    if (bundle.ethPrice.equals(ZERO_BD)) {
      trackedAmountETH = ZERO_BD
    } else {
      trackedAmountETH = trackedAmountUSD.div(bundle.ethPrice)
    }

    // update token0 global volume and token liquidity stats
    token0.tradeVolume = token0.tradeVolume.plus(amount0In.plus(amount0Out))
    token0.tradeVolumeUSD = token0.tradeVolumeUSD.plus(trackedAmountUSD)
    token0.untrackedVolumeUSD = token0.untrackedVolumeUSD.plus(derivedAmountUSD)

    // update token1 global volume and token liquidity stats
    token1.tradeVolume = token1.tradeVolume.plus(amount1In.plus(amount1Out))
    token1.tradeVolumeUSD = token1.tradeVolumeUSD.plus(trackedAmountUSD)
    token1.untrackedVolumeUSD = token1.untrackedVolumeUSD.plus(derivedAmountUSD)

    // update txn counts
    token0.txCount = token0.txCount.plus(ONE_BI)
    token1.txCount = token1.txCount.plus(ONE_BI)

    // update pair volume data, use tracked amount if we have it as its probably more accurate
    pair.volumeUSD = pair.volumeUSD.plus(trackedAmountUSD)
    pair.volumeToken0 = pair.volumeToken0.plus(amount0Total)
    pair.volumeToken1 = pair.volumeToken1.plus(amount1Total)
    pair.untrackedVolumeUSD = pair.untrackedVolumeUSD.plus(derivedAmountUSD)
    pair.txCount = pair.txCount.plus(ONE_BI)
    pair.save()

    // update global values, only used tracked amounts for volume
    const uniswap = UniswapFactory.load(FACTORY_ADDRESS)!
    uniswap.totalVolumeUSD = uniswap.totalVolumeUSD.plus(trackedAmountUSD)
    uniswap.totalVolumeETH = uniswap.totalVolumeETH.plus(trackedAmountETH)
    uniswap.untrackedVolumeUSD = uniswap.untrackedVolumeUSD.plus(derivedAmountUSD)
    uniswap.txCount = uniswap.txCount.plus(ONE_BI)

    // save entities
    pair.save()
    token0.save()
    token1.save()
    uniswap.save()

    let transaction = Transaction.load(event.transaction.hash.toHexString())
    if (transaction === null) {
      transaction = new Transaction(event.transaction.hash.toHexString())
      transaction.blockNumber = event.block.number
      transaction.timestamp = event.block.timestamp
      transaction.mints = []
      transaction.swaps = []
      transaction.burns = []
    }
    const swaps = transaction.swaps
    const swap = new SwapEvent(
      event.transaction.hash.toHexString().concat('-').concat(U256.fromI32(swaps.length).toString())
    )

    // update swap event
    swap.transaction = transaction.id
    swap.pair = pair.id
    swap.timestamp = transaction.timestamp
    swap.transaction = transaction.id
    swap.sender = event.params.sender
    swap.amount0In = amount0In
    swap.amount1In = amount1In
    swap.amount0Out = amount0Out
    swap.amount1Out = amount1Out
    swap.to = event.params.to
    swap.from = event.transaction.from
    swap.logIndex = event.logIndex
    // use the tracked amount if we have it
    swap.amountUSD = trackedAmountUSD === ZERO_BD ? derivedAmountUSD : trackedAmountUSD
    swap.save()

    // update the transaction

    // TODO: Consider using .concat() for handling array updates to protect
    // against unintended side effects for other code paths.
    swaps.push(swap.id)
    transaction.swaps = swaps
    transaction.save()

    // update day entities
    const pairDayData = updatePairDayData(event)
    const pairHourData = updatePairHourData(event)
    const uniswapDayData = updateUniswapDayData(event)
    const token0DayData = updateTokenDayData(token0 as Token, event)
    const token1DayData = updateTokenDayData(token1 as Token, event)

    // swap specific updating
    uniswapDayData.dailyVolumeUSD = uniswapDayData.dailyVolumeUSD.plus(trackedAmountUSD)
    uniswapDayData.dailyVolumeETH = uniswapDayData.dailyVolumeETH.plus(trackedAmountETH)
    uniswapDayData.dailyVolumeUntracked = uniswapDayData.dailyVolumeUntracked.plus(derivedAmountUSD)
    uniswapDayData.save()

    // swap specific updating for pair
    pairDayData.dailyVolumeToken0 = pairDayData.dailyVolumeToken0.plus(amount0Total)
    pairDayData.dailyVolumeToken1 = pairDayData.dailyVolumeToken1.plus(amount1Total)
    pairDayData.dailyVolumeUSD = pairDayData.dailyVolumeUSD.plus(trackedAmountUSD)
    pairDayData.save()

    // update hourly pair data
    pairHourData.hourlyVolumeToken0 = pairHourData.hourlyVolumeToken0.plus(amount0Total)
    pairHourData.hourlyVolumeToken1 = pairHourData.hourlyVolumeToken1.plus(amount1Total)
    pairHourData.hourlyVolumeUSD = pairHourData.hourlyVolumeUSD.plus(trackedAmountUSD)
    pairHourData.save()

    // swap specific updating for token0
    token0DayData.dailyVolumeToken = token0DayData.dailyVolumeToken.plus(amount0Total)
    token0DayData.dailyVolumeETH = token0DayData.dailyVolumeETH.plus(amount0Total.times(token0.derivedETH as UD256))
    token0DayData.dailyVolumeUSD = token0DayData.dailyVolumeUSD.plus(
      amount0Total.times(token0.derivedETH as UD256).times(bundle.ethPrice)
    )
    token0DayData.save()

    // swap specific updating
    token1DayData.dailyVolumeToken = token1DayData.dailyVolumeToken.plus(amount1Total)
    token1DayData.dailyVolumeETH = token1DayData.dailyVolumeETH.plus(amount1Total.times(token1.derivedETH as UD256))
    token1DayData.dailyVolumeUSD = token1DayData.dailyVolumeUSD.plus(
      amount1Total.times(token1.derivedETH as UD256).times(bundle.ethPrice)
    )
    token1DayData.save()
  }
   */
