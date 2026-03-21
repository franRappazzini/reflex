import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getCreateAssociatedTokenIdempotentInstructionAsync,
  getSyncNativeInstruction,
} from "@solana-program/token";
import { AccountRole, Address, Instruction, KeyPairSigner } from "@solana/kit";
import { SYSTEM_PROGRAM_ADDRESS, getTransferSolInstruction } from "@solana-program/system";
import { getConfigPda, getMarketPda, getMarketVaultPda, getTreasuryPda } from "../utils/pda";

import { Accounts } from "../utils/accounts";
import { Client } from "../utils/client";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { constants } from "../utils/constants";

export type CreateMarketParams = {
  id: string;
  /** Amount of incentive tokens to lock (in lamports). */
  amount: bigint;
  yesMint: KeyPairSigner;
  noMint: KeyPairSigner;
  /** SOL to wrap as WSOL in the briber's ATA. Defaults to 100 SOL. */
  briberWsolAmount?: bigint;
  /** Incentive mint, defaults to WSOL. */
  incentiveMint?: Address;
};

/**
 * Builds all instructions required for the `create_market` flow:
 *   1. Create the briber's WSOL ATA
 *   2. Transfer SOL into the ATA
 *   3. SyncNative (wrap SOL → WSOL)
 *   4. create_market program instruction
 *
 * Accounts (in order for the program ix):
 *   authority (readonly signer), config, briber (writable signer), market,
 *   incentive_mint, briber_ata, market_incentive_vault, treasury,
 *   outcome_yes_mint, outcome_no_mint, market_yes_vault, market_no_vault,
 *   token_program, associated_token_program, system_program
 */
export async function buildCreateMarketIxs(
  client: Client,
  accounts: Accounts,
  {
    id,
    amount,
    yesMint,
    noMint,
    briberWsolAmount = BigInt(100 * LAMPORTS_PER_SOL),
    incentiveMint = constants.WSOL_MINT,
  }: CreateMarketParams,
): Promise<Instruction[]> {
  const marketPda = await getMarketPda(id);

  const [briberAta] = await findAssociatedTokenPda({
    mint: incentiveMint,
    owner: accounts.briber.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const [configPda, wsolTreasuryPda, marketIncentiveVaultPda, marketYesVaultPda, marketNoVaultPda] =
    await Promise.all([
      getConfigPda(),
      getTreasuryPda(incentiveMint),
      getMarketVaultPda(marketPda, incentiveMint),
      getMarketVaultPda(marketPda, yesMint.address),
      getMarketVaultPda(marketPda, noMint.address),
    ]);

  const createBriberAtaIx = await getCreateAssociatedTokenIdempotentInstructionAsync({
    payer: accounts.briber,
    ata: briberAta,
    owner: accounts.briber.address,
    mint: incentiveMint,
  });

  const transferSolToAtaIx = getTransferSolInstruction({
    source: accounts.briber,
    destination: briberAta,
    amount: briberWsolAmount,
  });

  const syncNativeIx = getSyncNativeInstruction({ account: briberAta });

  // Layout: [u8 discriminator, u64 amount, ...utf8 id]
  const firstData = Buffer.alloc(9);
  firstData.writeUInt8(constants.CREATE_MARKET_DISCRIMINATOR, 0);
  firstData.writeBigUInt64LE(amount, 1);
  const ixData = Buffer.concat([firstData, Buffer.from(id, "utf8")]);

  const createMarketIx: Instruction = {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: client.wallet.address, role: AccountRole.READONLY_SIGNER },
      { address: configPda, role: AccountRole.READONLY },
      { address: accounts.briber.address, role: AccountRole.WRITABLE_SIGNER },
      { address: marketPda, role: AccountRole.WRITABLE },
      { address: incentiveMint, role: AccountRole.READONLY },
      { address: briberAta, role: AccountRole.WRITABLE },
      { address: marketIncentiveVaultPda, role: AccountRole.WRITABLE },
      { address: wsolTreasuryPda, role: AccountRole.WRITABLE },
      { address: yesMint.address, role: AccountRole.READONLY },
      { address: noMint.address, role: AccountRole.READONLY },
      { address: marketYesVaultPda, role: AccountRole.WRITABLE },
      { address: marketNoVaultPda, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };

  return [createBriberAtaIx, transferSolToAtaIx, syncNativeIx, createMarketIx];
}
