//! Protocol error codes.
//!
//! Every error is a u32 starting at 6000 (to avoid collisions with the
//! system program and token program error ranges). The error code is
//! returned as a ProgramError::Custom(code).
//!
//! The numbering is grouped by subsystem:
//!   6000–6019  general / config
//!   6020–6039  position / collateral
//!   6040–6059  peg / minting
//!   6060–6079  liquidation
//!   6080–6099  creatures

use pinocchio::program_error::ProgramError;

// ---- general ---------------------------------------------------------------

pub const ERR_INVALID_INSTRUCTION: u32 = 6000;
pub const ERR_INVALID_AUTHORITY: u32 = 6001;
pub const ERR_INVALID_PDA: u32 = 6002;
pub const ERR_ACCOUNT_NOT_WRITABLE: u32 = 6003;
pub const ERR_ACCOUNT_NOT_SIGNER: u32 = 6004;
pub const ERR_INVALID_DISCRIMINATOR: u32 = 6005;
pub const ERR_INVALID_OWNER: u32 = 6006;
pub const ERR_ALREADY_INITIALIZED: u32 = 6007;
pub const ERR_NOT_INITIALIZED: u32 = 6008;
pub const ERR_ARITHMETIC_OVERFLOW: u32 = 6009;
pub const ERR_INVALID_ACCOUNT_DATA: u32 = 6010;
pub const ERR_INVALID_PROGRAM_ID: u32 = 6011;

// ---- position / collateral -------------------------------------------------

pub const ERR_INSUFFICIENT_COLLATERAL: u32 = 6020;
pub const ERR_POSITION_NOT_EMPTY: u32 = 6021;
pub const ERR_POSITION_HAS_DEBT: u32 = 6022;
pub const ERR_WITHDRAW_EXCEEDS_SAFE: u32 = 6023;
pub const ERR_DEPOSIT_TOO_SMALL: u32 = 6024;
pub const ERR_POSITION_OWNER_MISMATCH: u32 = 6025;

// ---- peg / minting ---------------------------------------------------------

pub const ERR_MINT_EXCEEDS_CAPACITY: u32 = 6040;
pub const ERR_REDEEM_EXCEEDS_MINTED: u32 = 6041;
pub const ERR_BELOW_MIN_RATIO: u32 = 6042;
pub const ERR_ORACLE_STALE: u32 = 6043;
pub const ERR_ORACLE_PRICE_ZERO: u32 = 6044;
pub const ERR_INVALID_MINT: u32 = 6045;

// ---- liquidation -----------------------------------------------------------

pub const ERR_POSITION_HEALTHY: u32 = 6060;
pub const ERR_LIQUIDATION_TOO_LARGE: u32 = 6061;
pub const ERR_SELF_LIQUIDATION: u32 = 6062;

// ---- creatures -------------------------------------------------------------

pub const ERR_CREATURE_ALREADY_SPAWNED: u32 = 6080;
pub const ERR_NO_CREATURE: u32 = 6081;
pub const ERR_BELOW_SPAWN_THRESHOLD: u32 = 6082;
pub const ERR_CREATURE_OWNER_MISMATCH: u32 = 6083;
pub const ERR_MAX_GENERATION: u32 = 6084;
pub const ERR_INSUFFICIENT_XP: u32 = 6085;
pub const ERR_REROLL_FEE: u32 = 6086;

// ---- helper ----------------------------------------------------------------

/// Shorthand to return a custom error.
#[inline(always)]
pub fn err(code: u32) -> ProgramError {
    ProgramError::Custom(code)
}

/// Checked arithmetic helper — returns overflow error on None.
#[inline(always)]
pub fn checked_add(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_add(b).ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))
}

#[inline(always)]
pub fn checked_sub(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_sub(b).ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))
}

#[inline(always)]
pub fn checked_mul(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_mul(b).ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))
}

#[inline(always)]
pub fn checked_div(a: u64, b: u64) -> Result<u64, ProgramError> {
    a.checked_div(b).ok_or(ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))
}

/// Convert a u128 intermediate result back to u64 or overflow.
#[inline(always)]
pub fn narrow_u128(v: u128) -> Result<u64, ProgramError> {
    u64::try_from(v).map_err(|_| ProgramError::Custom(ERR_ARITHMETIC_OVERFLOW))
}
