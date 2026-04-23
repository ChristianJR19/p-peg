//! Instruction layout and zero-copy parsing.
//!
//! Every instruction is a packed byte buffer. The first byte is the
//! instruction index (0–10). The remaining bytes are the instruction
//! parameters laid out in little-endian order with no padding.
//!
//! We do NOT build a rust enum — that would allocate and copy. Instead
//! the processor reads the tag byte and hands the tail slice to the
//! appropriate handler, which reads its own fields directly.
//!
//! This file defines the tag constants and tiny reader helpers that
//! each handler uses to pull fields from the data slice.

use pinocchio::program_error::ProgramError;

use crate::error::{err, ERR_INVALID_INSTRUCTION};

// ---------------------------------------------------------------------------
// Instruction tags
// ---------------------------------------------------------------------------

pub const IX_INITIALIZE: u8 = 0;
pub const IX_DEPOSIT: u8 = 1;
pub const IX_WITHDRAW: u8 = 2;
pub const IX_MINT_PEGGED: u8 = 3;
pub const IX_REDEEM: u8 = 4;
pub const IX_LIQUIDATE: u8 = 5;
pub const IX_SPAWN_CREATURE: u8 = 6;
pub const IX_EVOLVE: u8 = 7;
pub const IX_REROLL: u8 = 8;
pub const IX_UPDATE_ORACLE: u8 = 9;
pub const IX_UPDATE_CONFIG: u8 = 10;

// ---------------------------------------------------------------------------
// Data readers — pull typed values from a byte slice at a given offset.
//
// These are intentionally minimal. Each one is a single bounds check
// plus a from_le_bytes call. No allocations, no copies beyond the
// final integer/bool value.
// ---------------------------------------------------------------------------

/// Read a u8 from `data` at `offset`.
#[inline(always)]
pub fn read_u8(data: &[u8], offset: usize) -> Result<u8, ProgramError> {
    data.get(offset).copied().ok_or(err(ERR_INVALID_INSTRUCTION))
}

/// Read a u16 from `data` at `offset` (little-endian).
#[inline(always)]
pub fn read_u16(data: &[u8], offset: usize) -> Result<u16, ProgramError> {
    let end = offset + 2;
    if end > data.len() {
        return Err(err(ERR_INVALID_INSTRUCTION));
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

/// Read a u64 from `data` at `offset` (little-endian).
#[inline(always)]
pub fn read_u64(data: &[u8], offset: usize) -> Result<u64, ProgramError> {
    let end = offset + 8;
    if end > data.len() {
        return Err(err(ERR_INVALID_INSTRUCTION));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[offset..end]);
    Ok(u64::from_le_bytes(buf))
}

/// Read a i64 from `data` at `offset` (little-endian).
#[inline(always)]
pub fn read_i64(data: &[u8], offset: usize) -> Result<i64, ProgramError> {
    let end = offset + 8;
    if end > data.len() {
        return Err(err(ERR_INVALID_INSTRUCTION));
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&data[offset..end]);
    Ok(i64::from_le_bytes(buf))
}

/// Read a 32-byte pubkey from `data` at `offset`.
#[inline(always)]
pub fn read_pubkey(data: &[u8], offset: usize) -> Result<[u8; 32], ProgramError> {
    let end = offset + 32;
    if end > data.len() {
        return Err(err(ERR_INVALID_INSTRUCTION));
    }
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[offset..end]);
    Ok(buf)
}

/// Read a bool from `data` at `offset` (0 = false, anything else = true).
#[inline(always)]
pub fn read_bool(data: &[u8], offset: usize) -> Result<bool, ProgramError> {
    read_u8(data, offset).map(|v| v != 0)
}

// ---------------------------------------------------------------------------
// Instruction data layouts (documented here, consumed in handlers)
// ---------------------------------------------------------------------------
//
// Initialize (IX 0):
//   [0..8]   min_collateral_ratio  u64
//   [8..16]  liquidation_bonus     u64
//   [16..24] spawn_threshold       u64
//   [24..32] reroll_fee            u64
//   [32..40] protocol_fee_bps      u64
//
// Deposit (IX 1):
//   [0..8]   amount                u64   lamports to deposit
//
// Withdraw (IX 2):
//   [0..8]   amount                u64   lamports to withdraw
//
// MintPegged (IX 3):
//   [0..8]   amount                u64   pUSD amount (6 decimals)
//
// Redeem (IX 4):
//   [0..8]   amount                u64   pUSD amount to burn
//
// Liquidate (IX 5):
//   [0..8]   repay_amount          u64   pUSD to repay on behalf of pos
//
// SpawnCreature (IX 6):
//   (no data — creature DNA derived on-chain)
//
// Evolve (IX 7):
//   [0..8]   feed_amount           u64   lamports to feed the creature
//
// Reroll (IX 8):
//   (no data — fee taken from position collateral)
//
// UpdateOracle (IX 9):
//   [0..8]   price                 u64   new price (6 decimals)
//   [8..16]  confidence            u64   confidence interval
//
// UpdateConfig (IX 10):
//   [0]      field_index           u8    which field to update
//   [1..9]   new_value             u64   new value for that field
//
//   field_index:
//     0 = min_collateral_ratio
//     1 = liquidation_bonus
//     2 = spawn_threshold
//     3 = reroll_fee
//     4 = protocol_fee_bps
