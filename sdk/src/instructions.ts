/**
 * Instruction builders for the p-peg protocol.
 *
 * Each function returns a TransactionInstruction that can be added
 * to a Transaction or VersionedTransaction.
 */

import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_SLOT_HASHES_PUBKEY,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import BN from "bn.js";
import {
  Instruction,
  PROGRAM_ID,
  InitializeParams,
  DepositParams,
  WithdrawParams,
  MintPeggedParams,
  RedeemParams,
  LiquidateParams,
  EvolveParams,
  UpdateOracleParams,
  UpdateConfigParams,
} from "./types";

// ---------------------------------------------------------------------------
// Data encoding helpers
// ---------------------------------------------------------------------------

function encodeU8(val: number): Buffer {
  const buf = Buffer.alloc(1);
  buf.writeUInt8(val, 0);
  return buf;
}

function encodeU64(val: BN): Buffer {
  return val.toArrayLike(Buffer, "le", 8);
}

function encodeInstruction(tag: Instruction, data: Buffer[]): Buffer {
  return Buffer.concat([encodeU8(tag), ...data]);
}

// ---------------------------------------------------------------------------
// Instruction builders
// ---------------------------------------------------------------------------

export function createInitializeInstruction(
  authority: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  oracle: PublicKey,
  peggedMint: PublicKey,
  params: InitializeParams = {},
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const defaults = {
    minCollateralRatio: new BN(15000),
    liquidationBonus: new BN(500),
    spawnThreshold: new BN(500_000_000),
    rerollFee: new BN(10_000_000),
    protocolFeeBps: new BN(30),
  };

  const p = { ...defaults, ...params };

  const data = encodeInstruction(Instruction.Initialize, [
    encodeU64(p.minCollateralRatio),
    encodeU64(p.liquidationBonus),
    encodeU64(p.spawnThreshold),
    encodeU64(p.rerollFee),
    encodeU64(p.protocolFeeBps),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: oracle, isSigner: false, isWritable: true },
      { pubkey: peggedMint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createDepositInstruction(
  depositor: PublicKey,
  position: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  params: DepositParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Deposit, [
    encodeU64(params.amount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: depositor, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createWithdrawInstruction(
  owner: PublicKey,
  position: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  oracle: PublicKey,
  params: WithdrawParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Withdraw, [
    encodeU64(params.amount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: oracle, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createMintPeggedInstruction(
  owner: PublicKey,
  position: PublicKey,
  config: PublicKey,
  oracle: PublicKey,
  peggedMint: PublicKey,
  ownerTokenAccount: PublicKey,
  mintAuthority: PublicKey,
  params: MintPeggedParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.MintPegged, [
    encodeU64(params.amount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: false },
      { pubkey: oracle, isSigner: false, isWritable: false },
      { pubkey: peggedMint, isSigner: false, isWritable: true },
      { pubkey: ownerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: mintAuthority, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createRedeemInstruction(
  owner: PublicKey,
  position: PublicKey,
  config: PublicKey,
  peggedMint: PublicKey,
  ownerTokenAccount: PublicKey,
  params: RedeemParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Redeem, [
    encodeU64(params.amount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: peggedMint, isSigner: false, isWritable: true },
      { pubkey: ownerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createLiquidateInstruction(
  liquidator: PublicKey,
  position: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  oracle: PublicKey,
  liquidatorTokenAccount: PublicKey,
  peggedMint: PublicKey,
  creature: PublicKey,
  params: LiquidateParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Liquidate, [
    encodeU64(params.repayAmount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: liquidator, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: oracle, isSigner: false, isWritable: false },
      { pubkey: liquidatorTokenAccount, isSigner: false, isWritable: true },
      { pubkey: peggedMint, isSigner: false, isWritable: true },
      { pubkey: creature, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createSpawnCreatureInstruction(
  owner: PublicKey,
  position: PublicKey,
  creature: PublicKey,
  config: PublicKey,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.SpawnCreature, []);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: creature, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createEvolveInstruction(
  owner: PublicKey,
  position: PublicKey,
  creature: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  params: EvolveParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Evolve, [
    encodeU64(params.feedAmount),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: creature, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createRerollInstruction(
  owner: PublicKey,
  position: PublicKey,
  creature: PublicKey,
  config: PublicKey,
  vault: PublicKey,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.Reroll, []);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: owner, isSigner: true, isWritable: true },
      { pubkey: position, isSigner: false, isWritable: true },
      { pubkey: creature, isSigner: false, isWritable: true },
      { pubkey: config, isSigner: false, isWritable: true },
      { pubkey: vault, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_SLOT_HASHES_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createUpdateOracleInstruction(
  authority: PublicKey,
  config: PublicKey,
  oracle: PublicKey,
  params: UpdateOracleParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const parts = [encodeU64(params.price)];
  if (params.confidence) {
    parts.push(encodeU64(params.confidence));
  }
  const data = encodeInstruction(Instruction.UpdateOracle, parts);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: config, isSigner: false, isWritable: false },
      { pubkey: oracle, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  });
}

export function createUpdateConfigInstruction(
  authority: PublicKey,
  config: PublicKey,
  params: UpdateConfigParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const data = encodeInstruction(Instruction.UpdateConfig, [
    encodeU8(params.fieldIndex),
    encodeU64(params.newValue),
  ]);

  return new TransactionInstruction({
    programId,
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: config, isSigner: false, isWritable: true },
    ],
    data,
  });
}
