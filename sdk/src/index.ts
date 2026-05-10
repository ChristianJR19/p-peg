/**
 * @ppeg/sdk — TypeScript SDK for the p-peg protocol.
 *
 * Usage:
 *   import { PPegClient, derivePosition, RARITY_NAMES } from "@ppeg/sdk";
 */

// Types and constants.
export * from "./types";

// PDA derivation.
export * from "./pda";

// Instruction builders.
export * from "./instructions";

// Account decoders and fetchers.
export * from "./accounts";

// Creature display helpers.
export * from "./creatures";

// High-level client.
export { PPegClient } from "./client";
export type { Wallet } from "./client";
