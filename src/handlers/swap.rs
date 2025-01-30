use alloy::{rpc::types::Log, sol, sol_types::SolEvent};

use crate::db::Database;

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

pub async fn handle_swaps(log: Log, db: &Database) {
    let event = Swap::decode_log(&log.inner, true).unwrap();
}
/*
export function handleSwap(event: Swap): void {
    const pair = Pair.load(event.address.toHexString())!
    const token0 = Token.load(pair.token0)
    const token1 = Token.load(pair.token1)
    if (token0 === null || token1 === null) {
      return
    }
    const amount0In = convertTokenToDecimal(event.params.amount0In, token0.decimals)
    const amount1In = convertTokenToDecimal(event.params.amount1In, token1.decimals)
    const amount0Out = convertTokenToDecimal(event.params.amount0Out, token0.decimals)
    const amount1Out = convertTokenToDecimal(event.params.amount1Out, token1.decimals)

    // totals for volume updates
    const amount0Total = amount0Out.plus(amount0In)
    const amount1Total = amount1Out.plus(amount1In)

    // ETH/USD prices
    const bundle = Bundle.load('1')!

    // get total amounts of derived USD and ETH for tracking
    const derivedAmountETH = token1.derivedETH
      .times(amount1Total)
      .plus(token0.derivedETH.times(amount0Total))
      .div(BigDecimal.fromString('2'))
    const derivedAmountUSD = derivedAmountETH.times(bundle.ethPrice)

    // only accounts for volume through white listed tokens
    const trackedAmountUSD = getTrackedVolumeUSD(
      amount0Total,
      token0 as Token,
      amount1Total,
      token1 as Token,
      pair as Pair
    )

    let trackedAmountETH: BigDecimal
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
      event.transaction.hash.toHexString().concat('-').concat(BigInt.fromI32(swaps.length).toString())
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
    token0DayData.dailyVolumeETH = token0DayData.dailyVolumeETH.plus(amount0Total.times(token0.derivedETH as BigDecimal))
    token0DayData.dailyVolumeUSD = token0DayData.dailyVolumeUSD.plus(
      amount0Total.times(token0.derivedETH as BigDecimal).times(bundle.ethPrice)
    )
    token0DayData.save()

    // swap specific updating
    token1DayData.dailyVolumeToken = token1DayData.dailyVolumeToken.plus(amount1Total)
    token1DayData.dailyVolumeETH = token1DayData.dailyVolumeETH.plus(amount1Total.times(token1.derivedETH as BigDecimal))
    token1DayData.dailyVolumeUSD = token1DayData.dailyVolumeUSD.plus(
      amount1Total.times(token1.derivedETH as BigDecimal).times(bundle.ethPrice)
    )
    token1DayData.save()
  }
   */
