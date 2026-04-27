//! Deposit SOL collateral into a position.
//!
//! If the position PDA does not exist yet, the caller must create it
//! before invoking this instruction (via a create_account ix in the
//! same transaction). This instruction assumes the account is already
//! allocated and either uninitialized or previously initialized.
//!
//! Accounts:
//!   0. [signer]    depositor
//!   1. [writable]  position PDA  (seeds: ["position", config, depositor])
//!   2. [writable]  config PDA
//!   3. [writable]  vault PDA
//!   4. []          system_program
//!   5. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, position, vault};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let depositor = &accounts[0];
    let position_ai = &accounts[1];
    let config_ai = &accounts[2];
    let vault_ai = &accounts[3];
    let system_prog = &accounts[4];
    let clock_ai = &accounts[5];

    if !depositor.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;

    let amount = read_u64(data, 0)?;
    if amount == 0 {
        return Err(err(ERR_DEPOSIT_TOO_SMALL));
    }

    // Verify position PDA.
    let pos_seeds: &[&[u8]] = &[POSITION_SEED, config_ai.key().as_ref(), depositor.key().as_ref()];
    let (expected_pos, pos_bump) = pinocchio::pubkey::find_program_address(pos_seeds, program_id);
    if *position_ai.key() != expected_pos {
        return Err(err(ERR_INVALID_PDA));
    }

    // Check if position is already initialized.
    let is_new = {
        let d = position_ai.try_borrow_data().map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
        d.len() < 8 || d[0..8] != DISC_POSITION
    };

    let now = vault::current_timestamp(clock_ai)?;

    if is_new {
        // Initialize position.
        position::write_discriminator(position_ai)?;
        position::set_owner(position_ai, depositor.key())?;
        position::set_collateral(position_ai, 0)?;
        position::set_minted(position_ai, 0)?;
        position::set_deposited_at(position_ai, now)?;
        position::set_last_interact(position_ai, now)?;
        position::set_has_creature(position_ai, false)?;
        position::set_bump(position_ai, pos_bump)?;

        // Zero out creature pubkey.
        let zero_key = Pubkey::default();
        position::set_creature(position_ai, &zero_key)?;
    } else {
        position::validate(position_ai)?;
        // Verify owner matches.
        let owner = position::owner(position_ai)?;
        if owner != *depositor.key() {
            return Err(err(ERR_POSITION_OWNER_MISMATCH));
        }
    }

    // Transfer SOL from depositor to vault.
    vault::transfer_to_vault(depositor, vault_ai, system_prog, amount)?;

    // Update position collateral.
    position::add_collateral(position_ai, amount)?;
    position::set_last_interact(position_ai, now)?;

    // Update global collateral counter.
    config::add_total_collateral(config_ai, amount)?;

    Ok(())
}
