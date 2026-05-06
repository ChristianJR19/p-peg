//! Evolve a creature by feeding it collateral.
//!
//! Feeding adds collateral to the parent position AND grants XP to
//! the creature. When the creature accumulates enough XP it evolves
//! to the next generation, which modifies its traits: species stays
//! the same, but power increases, rarity may improve, and mood shifts.
//!
//! This creates a game loop: deposit more collateral → creature grows
//! stronger → emotional attachment → keep position healthy → protocol
//! stability. DeFi as Tamagotchi.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. [writable]  creature PDA
//!   3. [writable]  config PDA
//!   4. [writable]  vault PDA
//!   5. []          system_program
//!   6. []          clock sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::engine::creature_gen;
use crate::error::*;
use crate::instruction::read_u64;
use crate::state::{config, creature, position, vault};

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> Result<(), ProgramError> {
    let owner_ai = &accounts[0];
    let position_ai = &accounts[1];
    let creature_ai = &accounts[2];
    let config_ai = &accounts[3];
    let vault_ai = &accounts[4];
    let system_prog = &accounts[5];
    let clock_ai = &accounts[6];

    if !owner_ai.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;
    creature::validate(creature_ai)?;

    let pos_owner = position::owner(position_ai)?;
    if pos_owner != *owner_ai.key() {
        return Err(err(ERR_POSITION_OWNER_MISMATCH));
    }

    let creature_owner = creature::owner(creature_ai)?;
    if creature_owner != *owner_ai.key() {
        return Err(err(ERR_CREATURE_OWNER_MISMATCH));
    }

    // Verify creature belongs to this position.
    let creature_pos = creature::position(creature_ai)?;
    if creature_pos != *position_ai.key() {
        return Err(err(ERR_INVALID_PDA));
    }

    let feed_amount = read_u64(data, 0)?;
    if feed_amount == 0 {
        return Err(err(ERR_DEPOSIT_TOO_SMALL));
    }

    let now = vault::current_timestamp(clock_ai)?;

    // Transfer SOL from owner to vault (this is also a deposit).
    vault::transfer_to_vault(owner_ai, vault_ai, system_prog, feed_amount)?;

    // Update position collateral.
    position::add_collateral(position_ai, feed_amount)?;
    position::set_last_interact(position_ai, now)?;

    // Update global collateral.
    config::add_total_collateral(config_ai, feed_amount)?;

    // Grant XP to creature.
    let new_xp = creature::add_xp(creature_ai, feed_amount)?;
    creature::increment_feeds(creature_ai)?;

    // Check if creature should evolve.
    let current_gen = creature::generation(creature_ai)?;
    let xp_threshold = checked_mul(XP_PER_EVOLUTION, (current_gen as u64) + 1)?;

    if new_xp >= xp_threshold && current_gen < MAX_GENERATION {
        // Evolve!
        let new_gen = creature::increment_generation(creature_ai)?;

        // Recalculate traits based on evolution.
        let dna = creature::dna(creature_ai)?;
        let collateral = position::collateral(position_ai)?;

        let new_power = creature_gen::evolved_power(dna, collateral, new_gen);
        let new_rarity = creature_gen::evolved_rarity(dna, collateral, new_gen);
        let new_mood = creature_gen::evolved_mood(dna, new_gen);

        creature::set_power(creature_ai, new_power)?;
        creature::set_rarity(creature_ai, new_rarity)?;
        creature::set_mood(creature_ai, new_mood)?;
        creature::set_evolved_at(creature_ai, now)?;
    }

    Ok(())
}
