# Peg Stability Mechanics

How p-peg maintains the pUSD peg through overcollateralization and liquidation.

## Overview

pUSD is a synthetic dollar-pegged stablecoin backed by SOL collateral. The protocol ensures every pUSD in circulation is backed by at least 150% worth of SOL (at the oracle price).

## Collateral Ratio

The collateral ratio is the value of deposited SOL divided by the amount of pUSD minted:

```
ratio = (collateral_lamports × price) / (minted_pusd × LAMPORTS_PER_SOL)
```

Expressed in basis points (15000 = 150%). The minimum ratio defaults to 15000 (150%) but is configurable by the protocol authority.

## Minting

When you mint pUSD, the protocol:

1. Checks that the oracle price is fresh (updated within 120 seconds).
2. Computes the maximum mintable amount: `max = collateral_value × BPS / min_ratio`.
3. Verifies the new total minted doesn't exceed the max.
4. Deducts a protocol fee (default 0.3%).
5. Mints pUSD to your token account via CPI.

## Redemption

Burning pUSD reduces your position's debt, freeing collateral for withdrawal. The protocol burns the tokens via CPI and updates the position state.

## Withdrawal

You can withdraw collateral as long as the position remains healthy after the withdrawal. The maximum withdrawable amount is:

```
min_collateral = minted × min_ratio × LAMPORTS / (price × BPS)
max_withdraw = current_collateral - min_collateral
```

## Liquidation

When a position's collateral ratio falls below the minimum (due to SOL price drop), anyone can liquidate it:

1. The liquidator specifies how much pUSD debt to repay.
2. The protocol burns that pUSD from the liquidator's token account.
3. The liquidator receives the equivalent SOL collateral plus a bonus (default 5%).
4. The position's collateral and debt are reduced accordingly.
5. If the position had a creature, the creature is destroyed.

### Liquidation Incentive

The 5% bonus makes liquidation profitable, ensuring positions are cleaned up quickly when prices move. The bonus is capped at the position's total collateral.

### Seize Calculation

```
base_seize = repay_amount × LAMPORTS / price
bonus = base_seize × liq_bonus_bps / BPS
total_seize = min(base_seize + bonus, position_collateral)
```

## Health Factor

The health factor is a normalized measure of position safety:

```
health = (collateral_ratio / min_ratio) × 10000
```

Values above 10000 (100%) are healthy. Values below 10000 are liquidatable.

## Oracle

The protocol uses a simple authority-controlled price feed. The oracle stores:
- Price (u64, 6 decimals — 150_000_000 = $150.00)
- Confidence interval (u64, 6 decimals)
- Last update timestamp

The oracle rejects reads if the price is stale (>120 seconds old) or zero. In production, this would be replaced by a Pyth or Switchboard integration.
