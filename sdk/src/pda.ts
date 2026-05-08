/**
 * PDA derivation helpers.
 *
 * Each function mirrors the on-chain seed layout so the SDK can
 * compute account addresses without calling the program.
 */

import { PublicKey } from "@solana/web3.js";
import {
  CONFIG_SEED,
  VAULT_SEED,
  POSITION_SEED,
  CREATURE_SEED,
  MINT_AUTH_SEED,
  ORACLE_SEED,
  PROGRAM_ID,
} from "./types";

/**
 * Derive the protocol config PDA.
 *
 * Seeds: ["config", authority]
 */
export function deriveConfig(
  authority: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [CONFIG_SEED, authority.toBuffer()],
    programId
  );
}

/**
 * Derive the collateral vault PDA.
 *
 * Seeds: ["vault", config]
 */
export function deriveVault(
  config: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [VAULT_SEED, config.toBuffer()],
    programId
  );
}

/**
 * Derive a user position PDA.
 *
 * Seeds: ["position", config, owner]
 */
export function derivePosition(
  config: PublicKey,
  owner: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [POSITION_SEED, config.toBuffer(), owner.toBuffer()],
    programId
  );
}

/**
 * Derive a creature PDA.
 *
 * Seeds: ["creature", position]
 */
export function deriveCreature(
  position: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [CREATURE_SEED, position.toBuffer()],
    programId
  );
}

/**
 * Derive the mint authority PDA.
 *
 * Seeds: ["mint_auth", config]
 */
export function deriveMintAuthority(
  config: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [MINT_AUTH_SEED, config.toBuffer()],
    programId
  );
}

/**
 * Derive the oracle price feed PDA.
 *
 * Seeds: ["oracle", config]
 */
export function deriveOracle(
  config: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [ORACLE_SEED, config.toBuffer()],
    programId
  );
}

/**
 * Derive all protocol PDAs from a single authority key.
 *
 * Convenience function for initialization.
 */
export function deriveAll(authority: PublicKey, programId: PublicKey = PROGRAM_ID) {
  const [config, configBump] = deriveConfig(authority, programId);
  const [vault, vaultBump] = deriveVault(config, programId);
  const [oracle, oracleBump] = deriveOracle(config, programId);
  const [mintAuth, mintAuthBump] = deriveMintAuthority(config, programId);

  return {
    config,
    configBump,
    vault,
    vaultBump,
    oracle,
    oracleBump,
    mintAuth,
    mintAuthBump,
  };
}
