//! Spawn a creature from a position.
//!
//! The position must have collateral above the spawn threshold and must
//! not already have a creature. The creature's DNA is derived
//! deterministically from the position pubkey, owner pubkey, slot hash,
//! and current timestamp — making each creature unique and verifiable.
//!
//! Accounts:
//!   0. [signer]    owner
//!   1. [writable]  position PDA
//!   2. [writable]  creature PDA (seeds: ["creature", position])
//!   3. []          config PDA
//!   4. []          system_program
//!   5. []          clock sysvar
//!   6. []          slot_hashes sysvar

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
    let _system = &accounts[4];
    let clock_ai = &accounts[5];
    let slot_hashes = &accounts[6];

    if !owner_ai.is_signer() {
        return Err(err(ERR_ACCOUNT_NOT_SIGNER));
    }

    config::validate(config_ai)?;
    position::validate(position_ai)?;

    let pos_owner = position::owner(position_ai)?;
    if pos_owner != *owner_ai.key() {
        return Err(err(ERR_POSITION_OWNER_MISMATCH));
    }

    // Check spawn prerequisites.
    let has = position::has_creature(position_ai)?;
    if has {
        return Err(err(ERR_CREATURE_ALREADY_SPAWNED));
    }

    let collateral = position::collateral(position_ai)?;
    let threshold = config::spawn_threshold(config_ai)?;
    if collateral < threshold {
        return Err(err(ERR_BELOW_SPAWN_THRESHOLD));
    }

    // Verify creature PDA.
    let creature_seeds: &[&[u8]] = &[CREATURE_SEED, position_ai.key().as_ref()];
    let (expected, creature_bump) =
        pinocchio::pubkey::find_program_address(creature_seeds, program_id);
    if *creature_ai.key() != expected {
        return Err(err(ERR_INVALID_PDA));
    }

    let now = vault::current_timestamp(clock_ai)?;

    // Derive DNA from on-chain entropy.
    // We mix: position_key + owner_key + timestamp + slot_hash bytes.
    let entropy = gather_entropy(position_ai.key(), owner_ai.key(), now, slot_hashes)?;
    let dna = creature_gen::derive_dna(entropy);

    // Decode traits from DNA.
    let species = creature_gen::dna_species(dna);
    let element = creature_gen::dna_element(dna);
    let rarity = creature_gen::dna_rarity(dna, collateral);
    let power = creature_gen::dna_power(dna, collateral);
    let mood = creature_gen::initial_mood(dna);

    // Write creature state.
    creature::write_discriminator(creature_ai)?;
    creature::set_owner(creature_ai, owner_ai.key())?;
    creature::set_position(creature_ai, position_ai.key())?;
    creature::set_dna(creature_ai, dna)?;
    creature::set_generation(creature_ai, 0)?;
    creature::set_species(creature_ai, species)?;
    creature::set_element(creature_ai, element)?;
    creature::set_rarity(creature_ai, rarity)?;
    creature::set_mood(creature_ai, mood)?;
    creature::set_power(creature_ai, power)?;
    creature::set_spawned_at(creature_ai, now)?;
    creature::set_evolved_at(creature_ai, now)?;
    creature::set_xp(creature_ai, 0)?;
    creature::set_feeds(creature_ai, 0)?;
    creature::set_bump(creature_ai, creature_bump)?;

    // Link creature to position.
    position::set_creature(position_ai, creature_ai.key())?;
    position::set_has_creature(position_ai, true)?;
    position::set_last_interact(position_ai, now)?;

    Ok(())
}

/// Gather entropy from on-chain sources.
///
/// We take 8 bytes from each source and XOR them together, then mix
/// with a simple hash cascade. This is NOT cryptographically secure —
/// it is sufficient for cosmetic trait generation where the stakes are
/// "my creature is a drake instead of a gremlin" rather than money.
fn gather_entropy(
    position_key: &Pubkey,
    owner_key: &Pubkey,
    timestamp: u64,
    slot_hashes: &AccountInfo,
) -> Result<[u8; 32], ProgramError> {
    let mut entropy = [0u8; 32];

    // Mix position key.
    let pk_bytes: &[u8] = position_key.as_ref();
    for i in 0..32 {
        entropy[i] ^= pk_bytes[i];
    }

    // Mix owner key.
    let ok_bytes: &[u8] = owner_key.as_ref();
    for i in 0..32 {
        entropy[i] ^= ok_bytes[i];
    }

    // Mix timestamp.
    let ts_bytes = timestamp.to_le_bytes();
    for i in 0..8 {
        entropy[i] ^= ts_bytes[i];
        entropy[i + 8] ^= ts_bytes[i].wrapping_mul(31);
        entropy[i + 16] ^= ts_bytes[i].wrapping_add(97);
        entropy[i + 24] ^= ts_bytes[i].rotate_left(3);
    }

    // Mix slot hashes (read first 32 bytes of the sysvar data).
    let sh_data = slot_hashes.try_borrow_data()
        .map_err(|_| err(ERR_INVALID_ACCOUNT_DATA))?;
    let sh_len = sh_data.len().min(32);
    for i in 0..sh_len {
        entropy[i] ^= sh_data[i];
    }

    // Simple mixing cascade — spread entropy across all bytes.
    for round in 0..4u8 {
        let mut acc: u8 = round.wrapping_mul(0x9e);
        for i in 0..32 {
            acc = acc.wrapping_add(entropy[i]).wrapping_mul(0x6d);
            entropy[i] = acc;
        }
    }

    Ok(entropy)
}
