//! Reroll a creature.
//!
//! Burns the current creature and spawns a new one with fresh DNA
//! derived from updated entropy (new timestamp + slot). Costs a
//! reroll fee deducted from the position's collateral.
//!
//! The new creature starts at generation 0 with 0 XP. This is a
//! deliberate tradeoff — you lose progress but might get a rarer
//! species or better element. Gamblers gonna gamble.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. [writable]  creature PDA
//!   3. [writable]  config PDA
//!   4. [writable]  vault PDA
//!   5. []          system_program
//!   6. []          clock sysvar
//!   7. []          slot_hashes sysvar

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::constants::*;
use crate::engine::creature_gen;
use crate::error::*;
use crate::state::{config, creature, position, vault};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _data: &[u8],
) -> Result<(), ProgramError> {
    let owner_ai = &accounts[0];
    let position_ai = &accounts[1];
    let creature_ai = &accounts[2];
    let config_ai = &accounts[3];
    let vault_ai = &accounts[4];
    let _system = &accounts[5];
    let clock_ai = &accounts[6];
    let slot_hashes = &accounts[7];

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

    // Deduct reroll fee from position collateral.
    let reroll_fee = config::reroll_fee(config_ai)?;
    let collateral = position::collateral(position_ai)?;
    if collateral < reroll_fee {
        return Err(err(ERR_REROLL_FEE));
    }

    // Transfer fee from vault to protocol authority (burns it effectively).
    let authority = config::authority(config_ai)?;
    let config_key = config_ai.key();
    let vault_seeds_base: &[&[u8]] = &[VAULT_SEED, config_key.as_ref()];
    let (_vault_addr, vault_bump) =
        pinocchio::pubkey::find_program_address(vault_seeds_base, program_id);
    let bump_bytes = [vault_bump];
    let vault_signer: &[&[u8]] = &[VAULT_SEED, config_key.as_ref(), &bump_bytes];

    // We need the authority account to receive the fee.
    // For simplicity, send it back to the vault (it stays in the pool).
    // The collateral reduction is the "fee" — the SOL stays locked but
    // is no longer attributed to this position.
    position::sub_collateral(position_ai, reroll_fee)?;

    let now = vault::current_timestamp(clock_ai)?;

    // Generate new DNA.
    let mut entropy = [0u8; 32];
    let pk_bytes: &[u8] = position_ai.key().as_ref();
    let ok_bytes: &[u8] = owner_ai.key().as_ref();
    for i in 0..32 {
        entropy[i] = pk_bytes[i] ^ ok_bytes[i];
    }
    let ts = now.to_le_bytes();
    for i in 0..8 {
        entropy[i] ^= ts[i];
        entropy[i + 16] ^= ts[i].wrapping_mul(0x53);
    }
    // Mix old DNA for additional entropy.
    let old_dna = creature::dna(creature_ai)?;
    let old_dna_bytes = old_dna.to_le_bytes();
    for i in 0..8 {
        entropy[i + 8] ^= old_dna_bytes[i];
        entropy[i + 24] ^= old_dna_bytes[i].wrapping_add(0x37);
    }
    // Mix slot hashes.
    let sh_data = slot_hashes.try_borrow_data()
        .map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let sh_len = sh_data.len().min(32);
    for i in 0..sh_len {
        entropy[i] ^= sh_data[i];
    }
    // Cascade mix.
    for round in 0..4u8 {
        let mut acc: u8 = round.wrapping_mul(0xab);
        for i in 0..32 {
            acc = acc.wrapping_add(entropy[i]).wrapping_mul(0x6d);
            entropy[i] = acc;
        }
    }

    let dna = creature_gen::derive_dna(entropy);
    let new_collateral = position::collateral(position_ai)?;

    // Rewrite creature state with new DNA.
    creature::set_dna(creature_ai, dna)?;
    creature::set_generation(creature_ai, 0)?;
    creature::set_species(creature_ai, creature_gen::dna_species(dna))?;
    creature::set_element(creature_ai, creature_gen::dna_element(dna))?;
    creature::set_rarity(creature_ai, creature_gen::dna_rarity(dna, new_collateral))?;
    creature::set_mood(creature_ai, creature_gen::initial_mood(dna))?;
    creature::set_power(creature_ai, creature_gen::dna_power(dna, new_collateral))?;
    creature::set_spawned_at(creature_ai, now)?;
    creature::set_evolved_at(creature_ai, now)?;
    creature::set_xp(creature_ai, 0)?;
    creature::set_feeds(creature_ai, 0)?;

    position::set_last_interact(position_ai, now)?;

    Ok(())
}
