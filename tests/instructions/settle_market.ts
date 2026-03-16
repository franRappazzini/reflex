import { AccountRole, Instruction } from "@solana/kit";
import { getConfigPda, getMarketPda } from "../utils/pda";

import { Client } from "../utils/client";
import { MarketResolution } from "../utils/fetch/market";
import { constants } from "../utils/constants";

export type SettleMarketParams = {
  id: string;
  /** 1 = Yes, 2 = No — use the MarketResolution enum. */
  resolution: MarketResolution.Yes | MarketResolution.No;
};

/**
 * Builds the `settle_market` instruction.
 *
 * Accounts (in order):
 *   authority (readonly signer), config (readonly), market (writable)
 *
 * Data layout: [u8 discriminator=5, u8 resolution, ...utf8 id]
 *
 * Only the authority stored in Config can call this instruction.
 */
export async function buildSettleMarketIx(
  client: Client,
  { id, resolution }: SettleMarketParams,
): Promise<Instruction> {
  const [configPda, marketPda] = await Promise.all([getConfigPda(), getMarketPda(id)]);

  // Layout: [u8 discriminator=5, u8 resolution, ...utf8 id]
  const ixData = Buffer.concat([
    Buffer.from([constants.SETTLE_MARKET_DISCRIMINATOR, resolution]),
    Buffer.from(id, "utf8"),
  ]);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: client.wallet.address, role: AccountRole.READONLY_SIGNER },
      { address: configPda, role: AccountRole.READONLY },
      { address: marketPda, role: AccountRole.WRITABLE },
    ],
    data: ixData,
  };
}
