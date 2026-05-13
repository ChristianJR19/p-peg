# Changelog

All notable changes to p-peg are documented in this file.

## [0.5.0] - 2026-05-14

### Added
- Protocol fee on minting (configurable, default 0.3%)
- Rarity XP multipliers — rarer creatures gain XP faster
- `UpdateConfig` instruction for live parameter changes
- Creature age score computation
- Rarity probability display in SDK
- `PPegClient` high-level client class

### Changed
- Creature power formula now factors in evolution generation bonus
- Rarity distribution shifts are now collateral-tiered (5/10/50/100 SOL)
- Oracle staleness window reduced from 300s to 120s

## [0.4.0] - 2026-05-10

### Added
- Reroll instruction — burn creature, generate new DNA
- Creature mood system (8 mood types, shifts on evolution)
- Effective power computation (base × rarity multiplier)
- SDK creature description generator

### Fixed
- Overflow in liquidation seize calculation for large positions
- Creature PDA derivation used wrong seed order

## [0.3.0] - 2026-05-06

### Added
- Creature engine — spawn, evolve, trait derivation from DNA
- 16 species, 8 elements, 6 rarity levels
- Evolution system with generation-based XP thresholds
- Creature death on liquidation
- SDK creature display helpers

## [0.2.0] - 2026-04-29

### Added
- Liquidation instruction with configurable bonus
- Health factor computation
- Max withdrawable calculation with conservative rounding
- Oracle staleness check
- TypeScript SDK with instruction builders and account decoders

## [0.1.0] - 2026-04-22

### Added
- Initial protocol: initialize, deposit, withdraw, mint, redeem
- Pinocchio-based zero-copy state management
- Config, Position, Oracle account layouts
- SOL vault PDA with signed transfers
- Basic peg stability math engine
- Protocol constants and error codes
