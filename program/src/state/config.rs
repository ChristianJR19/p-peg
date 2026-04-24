//! Protocol configuration account.
//!
//! Layout (all little-endian):
//!
//!   offset  size  field
//!   ------  ----  -----
//!    0       8    discriminator
//!    8      32    authority pubkey
//!   40      32    vault pubkey
//!   72      32    pegged_mint pubkey
//!  104       8    min_collateral_ratio (bps)
//!  112       8    liquidation_bonus (bps)
//!  120       8    spawn_threshold (lamports)
//!  128       8    reroll_fee (lamports)
//!  136       8    protocol_fee_bps
//!  144       8    total_collateral (lamports)
//!  152       8    total_minted (pUSD raw units)
//!  160       1    bump
//!  161       7    _reserved
//!  ------  ----
//!  total   168

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::constants::{CONFIG_SIZE, DISC_CONFIG};
use crate::error::{err, ERR_INVALID_DISCRIMINATOR, ERR_INVALID_ACCOUNT_DATA};

const OFF_DISC: usize = 0;
const OFF_AUTHORITY: usize = 8;
const OFF_VAULT: usize = 40;
const OFF_MINT: usize = 72;
const OFF_MIN_RATIO: usize = 104;
const OFF_LIQ_BONUS: usize = 112;
const OFF_SPAWN_THRESH: usize = 120;
const OFF_REROLL_FEE: usize = 128;
const OFF_PROTOCOL_FEE: usize = 136;
const OFF_TOTAL_COLL: usize = 144;
const OFF_TOTAL_MINTED: usize = 152;
const OFF_BUMP: usize = 160;

/// Verify the discriminator matches and return a reference to the raw data.
#[inline]
pub fn validate(account: &AccountInfo) -> Result<(), ProgramError> {
    let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    if data.len() < CONFIG_SIZE {
        return Err(err(ERR_INVALID_ACCOUNT_DATA));
    }
    if data[OFF_DISC..OFF_DISC + 8] != DISC_CONFIG {
        return Err(err(ERR_INVALID_DISCRIMINATOR));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Readers — borrow data, read field, drop borrow immediately.
// ---------------------------------------------------------------------------

macro_rules! read_pubkey_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<Pubkey, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            let mut buf = [0u8; 32];
            buf.copy_from_slice(&data[$offset..$offset + 32]);
            Ok(Pubkey::from(buf))
        }
    };
}

macro_rules! read_u64_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<u64, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            let mut buf = [0u8; 8];
            buf.copy_from_slice(&data[$offset..$offset + 8]);
            Ok(u64::from_le_bytes(buf))
        }
    };
}

macro_rules! read_u8_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo) -> Result<u8, ProgramError> {
            let data = account.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            Ok(data[$offset])
        }
    };
}

read_pubkey_field!(authority, OFF_AUTHORITY);
read_pubkey_field!(vault, OFF_VAULT);
read_pubkey_field!(pegged_mint, OFF_MINT);
read_u64_field!(min_collateral_ratio, OFF_MIN_RATIO);
read_u64_field!(liquidation_bonus, OFF_LIQ_BONUS);
read_u64_field!(spawn_threshold, OFF_SPAWN_THRESH);
read_u64_field!(reroll_fee, OFF_REROLL_FEE);
read_u64_field!(protocol_fee_bps, OFF_PROTOCOL_FEE);
read_u64_field!(total_collateral, OFF_TOTAL_COLL);
read_u64_field!(total_minted, OFF_TOTAL_MINTED);
read_u8_field!(bump, OFF_BUMP);

// ---------------------------------------------------------------------------
// Writers — borrow_mut data, write field, drop borrow.
// ---------------------------------------------------------------------------

macro_rules! write_pubkey_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: &Pubkey) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$offset..$offset + 32].copy_from_slice(val.as_ref());
            Ok(())
        }
    };
}

macro_rules! write_u64_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: u64) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$offset..$offset + 8].copy_from_slice(&val.to_le_bytes());
            Ok(())
        }
    };
}

macro_rules! write_u8_field {
    ($name:ident, $offset:expr) => {
        #[inline]
        pub fn $name(account: &AccountInfo, val: u8) -> Result<(), ProgramError> {
            let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            data[$offset] = val;
            Ok(())
        }
    };
}

/// Write the discriminator. Called once during initialization.
#[inline]
pub fn write_discriminator(account: &AccountInfo) -> Result<(), ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    data[OFF_DISC..OFF_DISC + 8].copy_from_slice(&DISC_CONFIG);
    Ok(())
}

write_pubkey_field!(set_authority, OFF_AUTHORITY);
write_pubkey_field!(set_vault, OFF_VAULT);
write_pubkey_field!(set_pegged_mint, OFF_MINT);
write_u64_field!(set_min_collateral_ratio, OFF_MIN_RATIO);
write_u64_field!(set_liquidation_bonus, OFF_LIQ_BONUS);
write_u64_field!(set_spawn_threshold, OFF_SPAWN_THRESH);
write_u64_field!(set_reroll_fee, OFF_REROLL_FEE);
write_u64_field!(set_protocol_fee_bps, OFF_PROTOCOL_FEE);
write_u64_field!(set_total_collateral, OFF_TOTAL_COLL);
write_u64_field!(set_total_minted, OFF_TOTAL_MINTED);
write_u8_field!(set_bump, OFF_BUMP);

// ---------------------------------------------------------------------------
// Atomic updaters — read, modify, write in one borrow.
// ---------------------------------------------------------------------------

/// Add `delta` to total_collateral.
#[inline]
pub fn add_total_collateral(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_TOTAL_COLL..OFF_TOTAL_COLL + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(delta).ok_or(err(crate::error::ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_TOTAL_COLL..OFF_TOTAL_COLL + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

/// Subtract `delta` from total_collateral.
#[inline]
pub fn sub_total_collateral(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_TOTAL_COLL..OFF_TOTAL_COLL + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_sub(delta).ok_or(err(crate::error::ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_TOTAL_COLL..OFF_TOTAL_COLL + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

/// Add `delta` to total_minted.
#[inline]
pub fn add_total_minted(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_TOTAL_MINTED..OFF_TOTAL_MINTED + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_add(delta).ok_or(err(crate::error::ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_TOTAL_MINTED..OFF_TOTAL_MINTED + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}

/// Subtract `delta` from total_minted.
#[inline]
pub fn sub_total_minted(account: &AccountInfo, delta: u64) -> Result<u64, ProgramError> {
    let mut data = account.try_borrow_mut_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[OFF_TOTAL_MINTED..OFF_TOTAL_MINTED + 8]);
    let current = u64::from_le_bytes(buf);
    let new = current.checked_sub(delta).ok_or(err(crate::error::ERR_ARITHMETIC_OVERFLOW))?;
    data[OFF_TOTAL_MINTED..OFF_TOTAL_MINTED + 8].copy_from_slice(&new.to_le_bytes());
    Ok(new)
}
