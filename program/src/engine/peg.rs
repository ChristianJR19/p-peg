//! Peg stability math.
//!
//! All the arithmetic that determines whether a position is healthy,
//! how much pUSD can be minted against collateral, and how liquidation
//! amounts are computed. Every function uses u128 intermediaries to
//! avoid overflow on large values, then narrows back to u64.
//!
//! Key invariant:
//!   collateral_value >= minted * min_ratio / BPS_DENOMINATOR
//!
//! Where collateral_value = collateral_lamports * price / LAMPORTS_PER_SOL
//! and price has PRICE_DECIMALS decimal places.

use pinocchio::program_error::ProgramError;

use crate::constants::{BPS_DENOMINATOR, LAMPORTS_PER_SOL, PRICE_DECIMALS};
use crate::error::{narrow_u128, ERR_ARITHMETIC_OVERFLOW};

/// Compute the USD value of a lamport amount at a given price.
///
/// Returns a value in pUSD units (6 decimals).
///
///   value = lamports * price / LAMPORTS_PER_SOL
///
/// Example: 1 SOL (1_000_000_000 lamports) at $150 (150_000_000):
///   = 1_000_000_000 * 150_000_000 / 1_000_000_000
///   = 150_000_000 (= $150.000000)
#[inline]
pub fn lamports_to_pusd(lamports: u64, price: u64) -> Result<u64, ProgramError> {
    let v = (lamports as u128)
        .checked_mul(price as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;
    let result = v / (LAMPORTS_PER_SOL as u128);
    narrow_u128(result)
}

/// Convert a pUSD amount back to lamports at the given price.
///
///   lamports = pusd_amount * LAMPORTS_PER_SOL / price
#[inline]
pub fn pusd_to_lamports(pusd_amount: u64, price: u64) -> Result<u64, ProgramError> {
    if price == 0 {
        return Err(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW));
    }
    let v = (pusd_amount as u128)
        .checked_mul(LAMPORTS_PER_SOL as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;
    let result = v / (price as u128);
    narrow_u128(result)
}

/// Check whether a position is healthy (above min collateral ratio).
///
///   healthy = (collateral_value * BPS_DENOMINATOR) >= (minted * min_ratio)
///
/// We rearrange to avoid division:
///   collateral_lamports * price * BPS_DENOMINATOR >= minted * min_ratio * LAMPORTS_PER_SOL
#[inline]
pub fn is_healthy(
    collateral_lamports: u64,
    minted: u64,
    price: u64,
    min_ratio_bps: u64,
) -> bool {
    if minted == 0 {
        return true;
    }

    let lhs = (collateral_lamports as u128)
        .saturating_mul(price as u128)
        .saturating_mul(BPS_DENOMINATOR as u128);

    let rhs = (minted as u128)
        .saturating_mul(min_ratio_bps as u128)
        .saturating_mul(LAMPORTS_PER_SOL as u128);

    lhs >= rhs
}

/// Compute the current collateral ratio in basis points.
///
///   ratio_bps = (collateral_value / minted) * BPS_DENOMINATOR
///             = (collateral * price * BPS_DENOMINATOR) / (minted * LAMPORTS_PER_SOL)
///
/// Returns u64::MAX if minted == 0 (infinitely collateralized).
#[inline]
pub fn collateral_ratio_bps(
    collateral_lamports: u64,
    minted: u64,
    price: u64,
) -> Result<u64, ProgramError> {
    if minted == 0 {
        return Ok(u64::MAX);
    }

    let numerator = (collateral_lamports as u128)
        .checked_mul(price as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;

    let denominator = (minted as u128)
        .checked_mul(LAMPORTS_PER_SOL as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;

    let ratio = numerator / denominator;
    narrow_u128(ratio)
}

/// Maximum pUSD that can be minted against the given collateral.
///
///   max_mint = collateral_value * BPS_DENOMINATOR / min_ratio
///            = (collateral * price / LAMPORTS_PER_SOL) * BPS_DENOMINATOR / min_ratio
#[inline]
pub fn max_mintable(
    collateral_lamports: u64,
    price: u64,
    min_ratio_bps: u64,
) -> Result<u64, ProgramError> {
    if min_ratio_bps == 0 {
        return Err(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW));
    }

    let collateral_value = lamports_to_pusd(collateral_lamports, price)?;
    let max = (collateral_value as u128)
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;
    let result = max / (min_ratio_bps as u128);
    narrow_u128(result)
}

/// Compute remaining mint capacity.
///
///   remaining = max_mintable - already_minted
///
/// Returns 0 if already over the limit (should not happen in normal operation).
#[inline]
pub fn remaining_capacity(
    collateral_lamports: u64,
    minted: u64,
    price: u64,
    min_ratio_bps: u64,
) -> Result<u64, ProgramError> {
    let max = max_mintable(collateral_lamports, price, min_ratio_bps)?;
    Ok(max.saturating_sub(minted))
}

/// Compute a fee in basis points.
///
///   fee = amount * fee_bps / BPS_DENOMINATOR
#[inline]
pub fn compute_fee(amount: u64, fee_bps: u64) -> Result<u64, ProgramError> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;
    let result = fee / (BPS_DENOMINATOR as u128);
    narrow_u128(result)
}

/// Compute the maximum lamports that can be withdrawn without
/// breaking the minimum collateral ratio.
///
///   min_collateral = minted * min_ratio * LAMPORTS_PER_SOL / (price * BPS_DENOMINATOR)
///   max_withdraw   = current_collateral - min_collateral
#[inline]
pub fn max_withdrawable(
    collateral_lamports: u64,
    minted: u64,
    price: u64,
    min_ratio_bps: u64,
) -> Result<u64, ProgramError> {
    if minted == 0 {
        return Ok(collateral_lamports);
    }
    if price == 0 {
        return Err(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW));
    }

    let min_collateral_num = (minted as u128)
        .checked_mul(min_ratio_bps as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?
        .checked_mul(LAMPORTS_PER_SOL as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;

    let min_collateral_den = (price as u128)
        .checked_mul(BPS_DENOMINATOR as u128)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;

    // Round up to be conservative.
    let min_collateral = (min_collateral_num + min_collateral_den - 1) / min_collateral_den;
    let min_collateral_u64 = narrow_u128(min_collateral)?;

    Ok(collateral_lamports.saturating_sub(min_collateral_u64))
}

/// Compute the liquidation seize amount (collateral + bonus).
///
///   base_seize = repay_amount * LAMPORTS_PER_SOL / price
///   bonus      = base_seize * liq_bonus_bps / BPS_DENOMINATOR
///   total      = base_seize + bonus
#[inline]
pub fn liquidation_seize(
    repay_amount: u64,
    price: u64,
    liq_bonus_bps: u64,
) -> Result<u64, ProgramError> {
    let base = pusd_to_lamports(repay_amount, price)?;
    let bonus = compute_fee(base, liq_bonus_bps)?;
    let total = (base as u128) + (bonus as u128);
    narrow_u128(total)
}

/// Compute the "health factor" as a percentage (100 = exactly at min ratio).
///
/// Values above 100 are healthy. Values below 100 are liquidatable.
/// Returns u64 percentage with 2 decimal places (10000 = 100.00%).
#[inline]
pub fn health_factor(
    collateral_lamports: u64,
    minted: u64,
    price: u64,
    min_ratio_bps: u64,
) -> Result<u64, ProgramError> {
    if minted == 0 {
        return Ok(u64::MAX);
    }

    let ratio = collateral_ratio_bps(collateral_lamports, minted, price)?;
    let factor = (ratio as u128)
        .checked_mul(10000)
        .ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))?;
    let result = factor / (min_ratio_bps as u128);
    narrow_u128(result)
}
