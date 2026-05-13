# p-peg

**Pinocchio-based peg stability protocol with on-chain creatures on Solana.**

P-Peg combines a compute-optimized peg stability module (deposit SOL → mint pUSD) with a creature engine — every collateralized position above the spawn threshold summons an on-chain creature whose DNA, species, element, and rarity are derived from position parameters and slot entropy.

Built entirely with [Pinocchio](https://github.com/anza-xyz/pinocchio) — zero external dependencies, zero-copy state access, `#![no_std]`.

---

## Architecture

```
program/src/
├── lib.rs              # entrypoint, #![no_std]
├── processor.rs        # instruction dispatch (jump table on u8 tag)
├── constants.rs        # all magic numbers in one place
├── error.rs            # error codes 6000–6099, checked math helpers
├── instruction.rs      # data layout docs + zero-copy readers
├── state/              # account layouts + zero-copy accessors
│   ├── config.rs       # protocol config PDA (168 bytes)
│   ├── position.rs     # user position PDA (112 bytes)
│   ├── creature.rs     # creature PDA (128 bytes)
│   ├── oracle.rs       # price feed PDA (40 bytes)
│   └── vault.rs        # SOL vault helpers
├── instructions/       # one handler per instruction (11 total)
│   ├── initialize.rs   # create config + vault + oracle
│   ├── deposit.rs      # deposit SOL collateral
│   ├── withdraw.rs     # withdraw SOL (health check)
│   ├── mint_pegged.rs  # mint pUSD via CPI
│   ├── redeem.rs       # burn pUSD, reduce debt
│   ├── liquidate.rs    # liquidate unhealthy position, kill creature
│   ├── spawn_creature.rs  # spawn creature from position entropy
│   ├── evolve.rs       # feed creature → XP → evolution
│   ├── reroll.rs       # burn creature, reroll new DNA
│   ├── update_oracle.rs   # authority updates price
│   └── update_config.rs   # authority updates params
└── engine/             # pure math, no account access
    ├── peg.rs          # collateral ratio, max mint, liquidation math
    ├── creature_gen.rs # DNA derivation, trait extraction, evolution
    └── rarity.rs       # rarity distribution computation

sdk/src/                # TypeScript SDK
├── types.ts            # constants, enums, interfaces
├── pda.ts              # PDA derivation helpers
├── instructions.ts     # transaction instruction builders
├── accounts.ts         # account deserialization + RPC fetchers
├── creatures.ts        # creature display, stats, description gen
├── client.ts           # high-level PPegClient class
└── index.ts            # barrel exports

tests/
├── peg_math.rs         # unit tests for peg stability math
├── creature_gen.rs     # unit tests for creature generation
└── state_serde.rs      # state serialization roundtrip tests
```

## How It Works

### Peg Stability Module

1. **Deposit** SOL collateral into a position PDA.
2. **Mint** pUSD (a 6-decimal SPL token) against collateral at the oracle price, limited by the minimum collateral ratio (default 150%).
3. **Redeem** pUSD to reduce debt and free collateral for withdrawal.
4. **Withdraw** collateral — must maintain the health ratio.
5. **Liquidate** — anyone can repay an undercollateralized position's debt and seize collateral + a 5% bonus.

### Creature Engine

Every position above the spawn threshold (default 0.5 SOL) can summon a creature:

- **DNA** is a u64 derived from position key ⊕ owner key ⊕ timestamp ⊕ slot hashes, mixed through a 4-round cascade.
- DNA encodes: species (16), element (8), mood (8), rarity seed, power seed, visual variant, personality hash.
- **Rarity** depends on both DNA randomness and collateral size — bigger bags spawn rarer creatures.
- **Evolution** happens when you feed the creature (add collateral). Each generation requires more XP. Max generation is 10.
- **Liquidation kills the creature.** This is the key incentive: keep your position healthy or your creature dies.
- **Reroll** burns the creature and spawns a new one with fresh DNA. Costs a fee from collateral.

### Rarity Distribution

| Rarity    | Base Odds | With 100 SOL |
|-----------|-----------|--------------|
| Common    | 53%       | 34%          |
| Uncommon  | 25%       | 25%          |
| Rare      | 13%       | 19%          |
| Epic      | 6%        | 13%          |
| Legendary | 3%        | 6%           |
| Mythic    | 0%        | 3%           |

## SDK Usage

```typescript
import { PPegClient } from "@ppeg/sdk";
import { Connection, Keypair } from "@solana/web3.js";
import BN from "bn.js";

const connection = new Connection("https://api.devnet.solana.com");
const wallet = loadWallet(); // your wallet adapter

const client = new PPegClient(connection, wallet);

// Deposit 2 SOL
await client.deposit(new BN(2_000_000_000));

// Mint $100 pUSD
await client.mintPegged(new BN(100_000_000), peggedMint);

// Spawn a creature
await client.spawnCreature();

// Feed and evolve
await client.evolve(new BN(500_000_000));

// Check your creature
const creature = await client.getCreature();
console.log(client.creatureTitle(creature));
// → "Rare Water Leviathan (Gen 2)"
```

## Build

```bash
# Program
cargo build-sbf

# SDK
cd sdk && npm install && npm run build

# Tests
cargo test
```

## Why Pinocchio?

The entire program uses zero external dependencies beyond `pinocchio`, `pinocchio-system`, and `pinocchio-token`. This means:

- **~96% CU reduction** compared to Anchor equivalents
- **Tiny binary** — no borsh, no solana-program, no std
- **Zero-copy everything** — we read/write directly into account data bytes
- **No deserialization overhead** — instruction data is parsed field-by-field from the raw slice

This is the same approach used by the [p-token program](https://github.com/febo/p-token) (SIMD-0266), which just launched on mainnet.

## License

MIT
