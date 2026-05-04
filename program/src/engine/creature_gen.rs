//! Creature generation engine.
//!
//! Creatures are deterministic. Given the same entropy, the same
//! creature is produced every time. This is important for verifiability:
//! anyone can re-derive a creature's traits from its DNA and confirm
//! that the on-chain state is correct.
//!
//! DNA is a u64 split into bit-fields:
//!
//!   bits  0–3   (4 bits)  species index      (0–15)
//!   bits  4–6   (3 bits)  element index       (0–7)
//!   bits  7–9   (3 bits)  base mood           (0–7)
//!   bits 10–15  (6 bits)  rarity seed         (0–63)
//!   bits 16–31  (16 bits) power seed          (0–65535)
//!   bits 32–47  (16 bits) visual variant seed
//!   bits 48–63  (16 bits) personality hash
//!
//! Rarity and power are also influenced by the position's collateral
//! at spawn time — bigger bags spawn rarer creatures.

use crate::constants::{MAX_POWER, NUM_ELEMENTS, NUM_SPECIES, LAMPORTS_PER_SOL};
use crate::engine::rarity;

/// Derive a u64 DNA value from 32 bytes of entropy.
///
/// We fold the 32-byte entropy into 8 bytes using XOR and multiplication,
/// then interpret as little-endian u64.
#[inline]
pub fn derive_dna(entropy: [u8; 32]) -> u64 {
    let mut dna_bytes = [0u8; 8];

    // Fold 32 bytes into 8 by XOR-ing 4 groups of 8.
    for chunk in 0..4 {
        let base = chunk * 8;
        for i in 0..8 {
            dna_bytes[i] ^= entropy[base + i];
        }
    }

    // Additional mixing — rotate and multiply.
    let mut acc: u8 = 0x5a;
    for i in 0..8 {
        acc = acc.wrapping_add(dna_bytes[i]).wrapping_mul(0x9e);
        dna_bytes[i] ^= acc;
    }

    u64::from_le_bytes(dna_bytes)
}

/// Extract species index from DNA (bits 0–3).
#[inline]
pub fn dna_species(dna: u64) -> u8 {
    (dna & 0xF) as u8 % NUM_SPECIES
}

/// Extract element index from DNA (bits 4–6).
#[inline]
pub fn dna_element(dna: u64) -> u8 {
    ((dna >> 4) & 0x7) as u8 % NUM_ELEMENTS
}

/// Extract initial mood from DNA (bits 7–9).
#[inline]
pub fn initial_mood(dna: u64) -> u8 {
    ((dna >> 7) & 0x7) as u8
}

/// Derive base rarity from DNA + collateral.
///
/// The rarity seed (bits 10–15) provides a random component, but
/// larger collateral shifts the distribution toward rarer outcomes.
///
/// Rarity levels: 0=common, 1=uncommon, 2=rare, 3=epic, 4=legendary, 5=mythic
#[inline]
pub fn dna_rarity(dna: u64, collateral_lamports: u64) -> u8 {
    let seed = ((dna >> 10) & 0x3F) as u8; // 0–63
    let sol_amount = collateral_lamports / LAMPORTS_PER_SOL;
    rarity::compute_rarity(seed, sol_amount)
}

/// Derive base power from DNA + collateral.
///
/// Power ranges from 0 to MAX_POWER. The power seed gives a base
/// value, and collateral scales it upward.
#[inline]
pub fn dna_power(dna: u64, collateral_lamports: u64) -> u16 {
    let power_seed = ((dna >> 16) & 0xFFFF) as u16;
    let sol_amount = collateral_lamports / LAMPORTS_PER_SOL;

    // Base power from DNA (0–999).
    let base = (power_seed % 1000) as u64;

    // Collateral bonus: sqrt(SOL) * 100, capped at 4000.
    let collateral_bonus = integer_sqrt(sol_amount).saturating_mul(100).min(4000);

    let total = base.saturating_add(collateral_bonus).min(MAX_POWER as u64);
    total as u16
}

/// Extract the visual variant seed (bits 32–47).
///
/// Used by the SDK to deterministically generate pixel art or
/// procedural visuals for the creature. Not stored on-chain
/// separately — derived from DNA on demand.
#[inline]
pub fn visual_variant(dna: u64) -> u16 {
    ((dna >> 32) & 0xFFFF) as u16
}

/// Extract the personality hash (bits 48–63).
///
/// Used by the creature description generator in the SDK to
/// produce unique flavor text for each creature.
#[inline]
pub fn personality_hash(dna: u64) -> u16 {
    ((dna >> 48) & 0xFFFF) as u16
}

// ---------------------------------------------------------------------------
// Evolution trait recalculation
// ---------------------------------------------------------------------------

/// Recalculate power after evolution.
///
/// Each generation adds a bonus:
///   new_power = base_power + (generation * 200) + sqrt(collateral_SOL) * 50
///
/// Capped at MAX_POWER.
#[inline]
pub fn evolved_power(dna: u64, collateral_lamports: u64, generation: u16) -> u16 {
    let base_power = dna_power(dna, collateral_lamports);
    let gen_bonus = (generation as u64).saturating_mul(200);
    let sol = collateral_lamports / LAMPORTS_PER_SOL;
    let coll_bonus = integer_sqrt(sol).saturating_mul(50);

    let total = (base_power as u64)
        .saturating_add(gen_bonus)
        .saturating_add(coll_bonus)
        .min(MAX_POWER as u64);
    total as u16
}

/// Recalculate rarity after evolution.
///
/// Every 3 generations, the creature has a chance to upgrade rarity.
/// The chance is influenced by collateral size.
#[inline]
pub fn evolved_rarity(dna: u64, collateral_lamports: u64, generation: u16) -> u8 {
    let base = dna_rarity(dna, collateral_lamports);

    // Every 3 generations, bump rarity by 1 (max 5 = mythic).
    let gen_bumps = generation / 3;
    let evolved = (base as u16).saturating_add(gen_bumps).min(5) as u8;
    evolved
}

/// Recalculate mood after evolution.
///
/// Evolution shifts the mood based on the generation number.
/// Even generations → "proud" (6), odd generations → cycles through others.
#[inline]
pub fn evolved_mood(dna: u64, generation: u16) -> u8 {
    if generation == 0 {
        return initial_mood(dna);
    }
    if generation % 2 == 0 {
        6 // proud
    } else {
        // Cycle through non-idle moods based on DNA + generation.
        let base_mood = initial_mood(dna) as u16;
        let shifted = base_mood.wrapping_add(generation) % 7;
        (shifted + 1) as u8 // skip "idle" (0)
    }
}

// ---------------------------------------------------------------------------
// Creature stat helpers
// ---------------------------------------------------------------------------

/// Compute the "effective power" of a creature, factoring in rarity.
///
///   effective = base_power * (1 + rarity * 0.15)
///
/// Rarity multipliers:
///   common(0)=1.00, uncommon(1)=1.15, rare(2)=1.30,
///   epic(3)=1.45, legendary(4)=1.60, mythic(5)=1.75
#[inline]
pub fn effective_power(base_power: u16, rarity_level: u8) -> u16 {
    let multiplier = 100u32 + (rarity_level as u32) * 15;
    let result = (base_power as u32) * multiplier / 100;
    result.min(MAX_POWER as u32) as u16
}

/// Compute the XP threshold for the next evolution.
///
///   threshold = XP_PER_EVOLUTION * (current_gen + 1)
///
/// Each generation requires more XP than the last.
#[inline]
pub fn next_evolution_xp(current_gen: u16) -> u64 {
    crate::constants::XP_PER_EVOLUTION
        .saturating_mul((current_gen as u64) + 1)
}

/// Check if a creature has enough XP to evolve.
#[inline]
pub fn can_evolve(xp: u64, current_gen: u16) -> bool {
    if current_gen >= crate::constants::MAX_GENERATION {
        return false;
    }
    xp >= next_evolution_xp(current_gen)
}

/// Compute an "age score" based on how long the creature has existed.
///
/// Older creatures are more valuable in the creature marketplace.
/// Score increases logarithmically with age.
///
///   age_score = log2(age_seconds / 3600 + 1) * 100
#[inline]
pub fn age_score(spawned_at: u64, now: u64) -> u16 {
    let age_seconds = now.saturating_sub(spawned_at);
    let age_hours = age_seconds / 3600;
    let log_val = log2_approx(age_hours + 1);
    (log_val * 100).min(MAX_POWER as u64) as u16
}

// ---------------------------------------------------------------------------
// Integer math helpers (no_std)
// ---------------------------------------------------------------------------

/// Integer square root via Newton's method.
#[inline]
fn integer_sqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Approximate log2 for positive integers.
#[inline]
fn log2_approx(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    63u64.saturating_sub(n.leading_zeros() as u64)
}
