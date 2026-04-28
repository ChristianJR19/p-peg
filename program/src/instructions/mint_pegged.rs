//! Mint pUSD against a collateralized position.
//!
//! The amount of pUSD that can be minted is limited by the collateral
//! value (SOL * oracle_price) divided by the minimum collateral ratio.
//! A protocol fee is deducted from the minted amount.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. []          config PDA
//!   3. []          oracle PDA
//!   4. [writable]  pegged_mint (the pUSD SPL token mint)
//!   5. [writable]  owner_token_account (pUSD ATA)
//!   6. []          mint_authority PDA
//!   7. []          token_program
//!   8. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::engine::peg;
use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, oracle, position, vault};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let owner_ai = &accounts[0];
    let position_ai = &accounts[1];
    let config_ai = &accounts[2];
    let oracle_ai = &accounts[3];
    let mint_ai = &accounts[4];
    let owner_ata = &accounts[5];
    let mint_auth_ai = &accounts[6];
    let _token_prog = &accounts[7];
    let clock_ai = &accounts[8];

    if !owner_ai.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;
    oracle::validate(oracle_ai)?;

    let pos_owner = position::owner(position_ai)?;
    if pos_owner != *owner_ai.key() {
        return Err(err(ERR_POSITION_OWNER_MISMATCH));
    }

    let now = vault::current_timestamp(clock_ai)?;
    let amount = read_u64(data, 0)?;
    if amount == 0 {
        return Err(err(ERR_MINT_EXCEEDS_CAPACITY));
    }

    let price = oracle::get_valid_price(oracle_ai, now)?;
    let min_ratio = config::min_collateral_ratio(config_ai)?;
    let collateral = position::collateral(position_ai)?;
    let current_minted = position::minted(position_ai)?;

    // Compute max mintable.
    let max_mint = peg::max_mintable(collateral, price, min_ratio)?;
    let new_total_minted = checked_add(current_minted, amount)?;
    if new_total_minted > max_mint {
        return Err(err(ERR_MINT_EXCEEDS_CAPACITY));
    }

    // Verify the position would remain healthy after minting.
    if !peg::is_healthy(collateral, new_total_minted, price, min_ratio) {
        return Err(err(ERR_BELOW_MIN_RATIO));
    }

    // Compute protocol fee.
    let fee_bps = config::protocol_fee_bps(config_ai)?;
    let fee = peg::compute_fee(amount, fee_bps)?;
    let mint_amount = checked_sub(amount, fee)?;

    // Mint pUSD to the owner's token account.
    // Derive mint authority PDA.
    let config_key = config_ai.key();
    let auth_seeds_base: &[&[u8]] = &[MINT_AUTH_SEED, config_key.as_ref()];
    let (_auth_addr, auth_bump) =
        pinocchio::pubkey::find_program_address(auth_seeds_base, program_id);
    let bump_bytes = [auth_bump];
    let auth_signer: &[&[u8]] = &[MINT_AUTH_SEED, config_key.as_ref(), &bump_bytes];

    pinocchio_token::instructions::MintTo {
        mint: mint_ai,
        account: owner_ata,
        mint_authority: mint_auth_ai,
    }
    .invoke_signed_with_args(mint_amount, &[auth_signer])?;

    // Update position debt.
    position::add_minted(position_ai, amount)?;
    position::set_last_interact(position_ai, now)?;

    // Update global minted counter.
    config::add_total_minted(config_ai, amount)?;

    Ok(())
}
