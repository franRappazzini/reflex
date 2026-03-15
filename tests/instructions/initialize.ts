import { AccountRole, Instruction } from "@solana/kit";
import { getConfigPda, getTreasuryPda } from "../utils/pda";

import { Client } from "../utils/client";
import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { constants } from "../utils/constants";

/**
 * Builds the `initialize` instruction.
 *
 * Accounts (in order):
 *   authority, config, wsol_mint, usdc_mint,
 *   wsol_treasury, usdc_treasury, token_program, system_program
 */
export async function buildInitializeIx(
  client: Client,
  feeBps: number = 500,
  briberFeeBps: number = 500,
): Promise<Instruction> {
  // Layout: [u8 discriminator, u16 fee_bps, u16 briber_fee_bps]
  const data = Buffer.alloc(5);
  data.writeUInt8(0, 0);
  data.writeUInt16LE(feeBps, 1);
  data.writeUInt16LE(briberFeeBps, 3);

  const [configPda, wsolTreasuryPda, usdcTreasuryPda] = await Promise.all([
    getConfigPda(),
    getTreasuryPda(constants.WSOL_MINT),
    getTreasuryPda(constants.USDC_MINT),
  ]);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: client.wallet.address, role: AccountRole.WRITABLE_SIGNER },
      { address: configPda, role: AccountRole.WRITABLE },
      { address: constants.WSOL_MINT, role: AccountRole.READONLY },
      { address: constants.USDC_MINT, role: AccountRole.READONLY },
      { address: wsolTreasuryPda, role: AccountRole.WRITABLE },
      { address: usdcTreasuryPda, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data,
  };
}
