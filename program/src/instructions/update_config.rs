//! Update protocol configuration parameters.
//!
//! Only the authority can update config. One field at a time to keep
//! the instruction data layout simple and the CU cost minimal.
//!
//! Accounts:
//!   0. [signer]    authority
//!   1. [writable]  config PDA

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::*;
use crate::instruction::{read_u8, read_u64};
use crate::state::config;

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let authority = &accounts[0];
    let config_ai = &accounts[1];

    if !authority.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;

    let expected_auth = config::authority(config_ai)?;
    if expected_auth != *authority.key() {
        return Err(err(ERR_INVALID_AUTHORITY));
    }

    let field_index = read_u8(data, 0)?;
    let new_value = read_u64(data, 1)?;

    match field_index {
        0 => config::set_min_collateral_ratio(config_ai, new_value)?,
        1 => config::set_liquidation_bonus(config_ai, new_value)?,
        2 => config::set_spawn_threshold(config_ai, new_value)?,
        3 => config::set_reroll_fee(config_ai, new_value)?,
        4 => config::set_protocol_fee_bps(config_ai, new_value)?,
        _ => return Err(err(ERR_INVALID_INSTRUCTION)),
    }

    Ok(())
}
