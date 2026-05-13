# Architecture

This document describes the internal architecture of p-peg for contributors and auditors.

## Design Principles

**No dependencies beyond pinocchio.** The program uses `#![no_std]` and relies solely on the pinocchio crate family. This eliminates borsh, solana-program, and any transitive dependency tree.

**Zero-copy everything.** State accounts are read and written directly via byte offsets. There are no intermediate Rust structs for account data — every field accessor borrows the account data slice, reads bytes at a fixed offset, and drops the borrow. This is cheaper in CU and memory than any deserialization framework.

**Instruction data is not deserialized.** The processor reads the first byte (tag), dispatches to the handler, and the handler reads its own fields from the remaining bytes with `read_u64`, `read_u8`, etc. No enum is constructed, no match on a deserialized type.

**Engine is pure math.** The `engine/` modules contain all the financial and creature-generation logic but never touch account data. This makes them independently testable and auditable.

## Account Layout

All accounts start with an 8-byte discriminator, followed by fields in a fixed layout. No variable-length data. No borsh length prefixes. Every account is a fixed size that can be determined at creation time.

### Config (168 bytes)

The singleton protocol configuration account, owned by the program. Stores protocol parameters, the authority pubkey, and running totals of collateral and minted pUSD.

PDA seeds: `["config", authority]`

### Position (112 bytes)

One per user per protocol instance. Tracks collateral deposited, pUSD minted, timestamps, and a link to the user's creature (if any).

PDA seeds: `["position", config, owner]`

### Creature (128 bytes)

One per position (optional). Stores DNA, decoded traits, generation, XP, and feed count. Creatures are deterministic — given the DNA, all traits can be re-derived.

PDA seeds: `["creature", position]`

### Oracle (40 bytes)

Simple price feed controlled by the protocol authority. Stores price (6 decimals), confidence interval, and last update timestamp. Designed to be swapped for Pyth/Switchboard in production.

PDA seeds: `["oracle", config]`

### Vault (system account)

A system-owned PDA holding all deposited SOL. Has no custom data — its lamport balance IS the collateral pool. The PDA seeds are used to sign outbound transfers.

PDA seeds: `["vault", config]`

## Instruction Flow

Every instruction follows the same pattern:

1. Unpack accounts from the `accounts` slice by index.
2. Verify signer, ownership, and PDA derivation.
3. Validate discriminators on all state accounts.
4. Read instruction parameters from the data slice.
5. Execute business logic (often delegating to `engine/`).
6. Write state changes.
7. Execute CPIs (transfers, mints, burns) if needed.

## CPI Pattern

SOL transfers use `pinocchio_system::instructions::Transfer`. Token operations (mint, burn) use `pinocchio_token::instructions::MintTo` and `Burn`. All CPI calls are signed by the appropriate PDA (vault for SOL transfers, mint_authority for token operations).

## Security Model

- **Authority-gated**: only the protocol authority can update the oracle and config.
- **Self-liquidation prevention**: a position owner cannot liquidate their own position.
- **Oracle staleness**: the oracle price is rejected if it hasn't been updated within 120 seconds.
- **Checked arithmetic**: all math uses checked operations that return errors on overflow.
- **Health checks**: every withdraw and mint operation verifies the position remains above the minimum collateral ratio.
- **Creature death on liquidation**: if a position is liquidated, its creature account data is zeroed. This is irreversible.
