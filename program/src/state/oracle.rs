//! Oracle price feed account.
//!
//! A simple authority-controlled price feed. In production you would
//! integrate Pyth or Switchboard, but the protocol is oracle-agnostic
//! by design — it only reads price and confidence from this account.
//! Swapping the oracle backend is a one-line change in the config.
//!
//! Layout:
//!
//!   offset  size  field
//!   ------  ----  -----
//!    0       8    discriminator
//!    8       8    price (u64, 6 decimals — 150_000_000 = $150)
//!   16       8    confidence (u64, 6 decimals — price ± confidence)
//!   24       8    updated_at (unix timestamp)
//!   32       1    bump
//!   33       7    _reserved
//!  ------  ----
//!  total    40

use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

use crate::constants::DISC_ORACLE;
use crate::error::{err, ERR_INVALID_DISCRIMINATOR, ERR_INVALID_ACCOUNT_DATA};

const ORACLE_ACCOUNT_SIZE: usize = 40;

const OFF_DISC: usize = 0;
const OFF_PRICE: usize = 8;
const OFF_CONFIDENCE: usize = 16;
const OFF_UPDATED_AT: usize = 24;
const OFF_BUMP: usize = 32;

pub const ACCOUNT_SIZE: usize = ORACLE_ACCOUNT_SIZE;

/// Maximum age of a price update before it is considered stale (120 seconds).
pub const MAX_STALENESS: u64 = 120;

#[inline]
pub fn validate(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    if data.len() < ORACLE_ACCOUNT_SIZE {
        return Err(err(ERR_INVALID_ACCOUNT_DATA));
    }
    if data[OFF_DISC..OFF_DISC + 8] != DISC_ORACLE {
        return Err(err(ERR_INVALID_DISCRIMINATOR));
    }
    Ok(())
}

#[inline]
pub fn price(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_PRICE..OFF_PRICE + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn confidence(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_CONFIDENCE..OFF_CONFIDENCE + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn updated_at(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_UPDATED_AT..OFF_UPDATED_AT + 8]);
    Ok(u64::from_le_bytes(buf))
}

#[inline]
pub fn bump(account: &AccountInfo) -> Result<u8, ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    Ok(data[OFF_BUMP])
}

// ---- writers ---------------------------------------------------------------

#[inline]
pub fn write_discriminator(account: &AccountInfo) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_DISC..OFF_DISC + 8].copy_from_slice(&DISC_ORACLE);
    Ok(())
}

#[inline]
pub fn set_price(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_PRICE..OFF_PRICE + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_confidence(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_CONFIDENCE..OFF_CONFIDENCE + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_updated_at(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_UPDATED_AT..OFF_UPDATED_AT + 8].copy_from_slice(&val.to_le_bytes());
    Ok(())
}

#[inline]
pub fn set_bump(account: &AccountInfo, val: u8) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_BUMP] = val;
    Ok(())
}

/// Check that the oracle price is fresh enough to use for minting or
/// liquidation. Returns the price if valid.
#[inline]
pub fn get_valid_price(account: &AccountInfo, clock_ts: u64) -> Result<u64, ProgramError> {
    let p = price(account)?;
    if p == 0 {
        return Err(err(crate::error::ERR_ORACLE_PRICE_ZERO));
    }
    let updated = updated_at(account)?;
    if clock_ts.saturating_sub(updated) > MAX_STALENESS {
        return Err(err(crate::error::ERR_ORACLE_STALE));
    }
    Ok(p)
}
