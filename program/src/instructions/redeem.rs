//! Redeem pUSD and reduce position debt.
//!
//! Burns the specified amount of pUSD from the owner's token account
//! and reduces the minted amount on the position. This frees up
//! collateral capacity for withdrawal.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. [writable]  config PDA
//!   3. [writable]  pegged_mint
//!   4. [writable]  owner_token_account (pUSD ATA)
//!   5. []          token_program
//!   6. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, position, vault};

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let owner_ai = &accounts[0];
    let position_ai = &accounts[1];
    let config_ai = &accounts[2];
    let mint_ai = &accounts[3];
    let owner_ata = &accounts[4];
    let _token_prog = &accounts[5];
    let clock_ai = &accounts[6];

    if !owner_ai.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;

    let pos_owner = position::owner(position_ai)?;
    if pos_owner != *owner_ai.key() {
        return Err(err(ERR_POSITION_OWNER_MISMATCH));
    }

    let amount = read_u64(data, 0)?;
    let current_minted = position::minted(position_ai)?;
    if amount > current_minted {
        return Err(err(ERR_REDEEM_EXCEEDS_MINTED));
    }

    // Burn pUSD from the owner's token account.
    pinocchio_token::instructions::Burn {
        account: owner_ata,
        mint: mint_ai,
        authority: owner_ai,
    }
    .invoke_with_args(amount)?;

    let now = vault::current_timestamp(clock_ai)?;

    // Reduce position debt.
    position::sub_minted(position_ai, amount)?;
    position::set_last_interact(position_ai, now)?;

    // Reduce global minted counter.
    config::sub_total_minted(config_ai, amount)?;

    Ok(())
}
