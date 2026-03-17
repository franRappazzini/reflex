import { AccountRole, Instruction } from "@solana/kit";
import {
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getCreateAssociatedTokenInstructionAsync,
} from "@solana-program/token";
import { getConfigPda, getTreasuryPda } from "../utils/pda";

import { Client } from "../utils/client";
import { constants } from "../utils/constants";

/**
 * Builds the instructions required for the `withdraw_treasury` flow:
 *   1. (optional) Create authority's WSOL ATA if it doesn't exist yet.
 *   2. (optional) Create authority's USDC ATA if it doesn't exist yet.
 *   3. withdraw_treasury program instruction.
 *
 * Accounts (in order for the program ix):
 *   authority (writable signer), config, wsol_mint, usdc_mint,
 *   wsol_treasury, usdc_treasury,
 *   wsol_destination (authority ATA for WSOL),
 *   usdc_destination (authority ATA for USDC),
 *   token_program
 *
 * Data layout: [u8 discriminator=6]  — no extra fields.
 */
export async function buildWithdrawTreasuryIxs(client: Client): Promise<Instruction[]> {
  const [configPda, wsolTreasuryPda, usdcTreasuryPda] = await Promise.all([
    getConfigPda(),
    getTreasuryPda(constants.WSOL_MINT),
    getTreasuryPda(constants.USDC_MINT),
  ]);

  const [[wsolDestination], [usdcDestination]] = await Promise.all([
    findAssociatedTokenPda({
      mint: constants.WSOL_MINT,
      owner: client.wallet.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    }),
    findAssociatedTokenPda({
      mint: constants.USDC_MINT,
      owner: client.wallet.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    }),
  ]);

  const [createWsolAtaIx, createUsdcAtaIx] = await Promise.all([
    getCreateAssociatedTokenInstructionAsync({
      payer: client.wallet,
      ata: wsolDestination,
      owner: client.wallet.address,
      mint: constants.WSOL_MINT,
    }),
    getCreateAssociatedTokenInstructionAsync({
      payer: client.wallet,
      ata: usdcDestination,
      owner: client.wallet.address,
      mint: constants.USDC_MINT,
    }),
  ]);

  // Layout: [u8 discriminator=6]
  const ixData = Buffer.from([constants.WITHDRAW_TREASURY_DISCRIMINATOR]);

  const withdrawIx: Instruction = {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: client.wallet.address, role: AccountRole.WRITABLE_SIGNER },
      { address: configPda, role: AccountRole.READONLY },
      { address: constants.WSOL_MINT, role: AccountRole.READONLY },
      { address: constants.USDC_MINT, role: AccountRole.READONLY },
      { address: wsolTreasuryPda, role: AccountRole.WRITABLE },
      { address: usdcTreasuryPda, role: AccountRole.WRITABLE },
      { address: wsolDestination, role: AccountRole.WRITABLE },
      { address: usdcDestination, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };

  return [createWsolAtaIx, createUsdcAtaIx, withdrawIx];
}
