//! Collateral vault account.
//!
//! The vault is a system-owned PDA that holds all deposited SOL. It
//! has no custom data layout — it is a plain system account whose
//! lamport balance IS the collateral pool. The PDA seeds are used to
//! sign transfers out of the vault during withdrawals and liquidations.
//!
//! This module provides helpers for transferring lamports in and out
//! of the vault using CPI to the system program.

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::clock::Clock,
};

use crate::constants::VAULT_SEED;
use crate::error::{err, ERR_INVALID_PDA};

/// Derive the vault PDA address and bump.
///
/// Seeds: ["vault", config_pubkey]
#[inline]
pub fn derive_address(program_id: &[u8; 32], config_key: &[u8; 32]) -> ([u8; 32], u8) {
    let seeds: &[&[u8]] = &[VAULT_SEED, config_key];
    // pinocchio::pubkey::find_program_address returns (Pubkey, u8)
    pinocchio::pubkey::find_program_address(seeds, &pinocchio::pubkey::Pubkey::from(*program_id))
}

/// Transfer lamports from a signer account into the vault.
///
/// This is a simple SOL transfer — the signer pays lamports directly
/// to the vault PDA. We use the system program transfer instruction.
#[inline]
pub fn transfer_to_vault<'a>(
    from: &AccountInfo<'a>,
    vault: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    pinocchio_system::instructions::Transfer {
        from,
        to: vault,
    }
    .invoke_with_args(amount)?;
    Ok(())
}

/// Transfer lamports from the vault PDA to a destination account.
///
/// Requires the vault PDA signer seeds.
#[inline]
pub fn transfer_from_vault<'a>(
    vault: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
    signer_seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    pinocchio_system::instructions::Transfer {
        from: vault,
        to,
    }
    .invoke_signed_with_args(amount, &[signer_seeds])?;
    Ok(())
}

/// Get the current unix timestamp from the clock sysvar.
///
/// We read the clock from the account passed by the client rather
/// than using sol_get_clock_sysvar (which costs more CU on some
/// runtimes). The caller must pass the clock sysvar account.
#[inline]
pub fn current_timestamp(clock_account: &AccountInfo) -> Result<u64, ProgramError> {
    let clock = Clock::from_account_info(clock_account)?;
    // Clock.unix_timestamp is i64 — cast to u64 (negative is impossible
    // in practice but we saturate to 0 for safety).
    Ok(clock.unix_timestamp.max(0) as u64)
}
