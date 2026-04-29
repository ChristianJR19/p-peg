//! Liquidate an undercollateralized position.
//!
//! Anyone can liquidate a position whose collateral ratio has fallen
//! below the minimum. The liquidator repays some or all of the
//! position's pUSD debt and receives the equivalent SOL collateral
//! plus a liquidation bonus.
//!
//! If the position has a creature, the creature is killed (its account
//! data is zeroed). Liquidation is the only way creatures die. This
//! creates an emotional incentive to keep positions healthy.
//!
//! Accounts:
//!   0. [signer]    liquidator
//!   1. [writable]  position PDA (the unhealthy position)
//!   2. [writable]  config PDA
//!   3. [writable]  vault PDA
//!   4. []          oracle PDA
//!   5. [writable]  liquidator_token_account (pUSD ATA)
//!   6. [writable]  pegged_mint
//!   7. [writable]  creature PDA (pass system program if no creature)
//!   8. []          token_program
//!   9. []          system_program
//!  10. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::engine::peg;
use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, creature, oracle, position, vault};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let liquidator = &accounts[0];
    let position_ai = &accounts[1];
    let config_ai = &accounts[2];
    let vault_ai = &accounts[3];
    let oracle_ai = &accounts[4];
    let liq_ata = &accounts[5];
    let mint_ai = &accounts[6];
    let creature_ai = &accounts[7];
    let _token_prog = &accounts[8];
    let _system = &accounts[9];
    let clock_ai = &accounts[10];

    if !liquidator.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;
    oracle::validate(oracle_ai)?;

    // Prevent self-liquidation.
    let pos_owner = position::owner(position_ai)?;
    if pos_owner == *liquidator.key() {
        return Err(err(ERR_SELF_LIQUIDATION));
    }

    let now = vault::current_timestamp(clock_ai)?;
    let price = oracle::get_valid_price(oracle_ai, now)?;
    let min_ratio = config::min_collateral_ratio(config_ai)?;
    let liq_bonus_bps = config::liquidation_bonus(config_ai)?;

    let collateral = position::collateral(position_ai)?;
    let minted = position::minted(position_ai)?;

    // Verify position is actually unhealthy.
    if peg::is_healthy(collateral, minted, price, min_ratio) {
        return Err(err(ERR_POSITION_HEALTHY));
    }

    let repay_amount = read_u64(data, 0)?;
    if repay_amount == 0 || repay_amount > minted {
        return Err(err(ERR_LIQUIDATION_TOO_LARGE));
    }

    // Compute how much collateral the liquidator receives.
    // collateral_value = repay_amount / price * LAMPORTS_PER_SOL
    // bonus = collateral_value * liq_bonus_bps / BPS_DENOMINATOR
    let collateral_value = peg::pusd_to_lamports(repay_amount, price)?;
    let bonus = checked_mul(collateral_value, liq_bonus_bps)
        .and_then(|v| checked_div(v, BPS_DENOMINATOR))?;
    let total_seize = checked_add(collateral_value, bonus)?;

    // Cap seizure at the position's actual collateral.
    let actual_seize = if total_seize > collateral { collateral } else { total_seize };

    // Burn liquidator's pUSD.
    pinocchio_token::instructions::Burn {
        account: liq_ata,
        mint: mint_ai,
        authority: liquidator,
    }
    .invoke_with_args(repay_amount)?;

    // Transfer seized collateral from vault to liquidator.
    let config_key = config_ai.key();
    let vault_seeds_base: &[&[u8]] = &[VAULT_SEED, config_key.as_ref()];
    let (_vault_addr, vault_bump) =
        pinocchio::pubkey::find_program_address(vault_seeds_base, program_id);
    let bump_bytes = [vault_bump];
    let vault_signer: &[&[u8]] = &[VAULT_SEED, config_key.as_ref(), &bump_bytes];

    vault::transfer_from_vault(vault_ai, liquidator, actual_seize, vault_signer)?;

    // Update position.
    position::sub_collateral(position_ai, actual_seize)?;
    position::sub_minted(position_ai, repay_amount)?;
    position::set_last_interact(position_ai, now)?;

    // Update global counters.
    config::sub_total_collateral(config_ai, actual_seize)?;
    config::sub_total_minted(config_ai, repay_amount)?;

    // Kill the creature if the position has one.
    let has_creature = position::has_creature(position_ai)?;
    if has_creature {
        // Zero out the creature account data — the creature dies.
        let creature_data_len = {
            let d = creature_ai.try_borrow_data()
                .map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            d.len()
        };
        if creature_data_len >= 8 {
            let mut d = creature_ai.try_borrow_mut_data()
                .map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
            for byte in d.iter_mut().take(creature_data_len) {
                *byte = 0;
            }
        }

        // Clear creature reference on the position.
        let zero_key = Pubkey::default();
        position::set_creature(position_ai, &zero_key)?;
        position::set_has_creature(position_ai, false)?;
    }

    Ok(())
}
