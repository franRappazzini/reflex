import { AccountRole, Address, Instruction } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import { getMarketPda, getMarketVaultPda } from "../utils/pda";

import { Accounts } from "../utils/accounts";
import { constants } from "../utils/constants";

export type CancelMarketParams = {
  id: string;
  /** Address of the yes-outcome mint used when the market was created. */
  yesMint: Address;
  /** Address of the no-outcome mint used when the market was created. */
  noMint: Address;
};

/**
 * Builds the `cancel_market` instruction.
 *
 * Accounts (in order):
 *   briber (writable signer), market, incentive_mint,
 *   briber_ata, market_incentive_vault,
 *   market_yes_vault, market_no_vault
 *
 * Data layout: [u8 discriminator=3, ...utf8 id]
 *
 * Pre-condition: the market must have no pending fees
 * (available_yes_fees == 0 && available_no_fees == 0).
 */
export async function buildCancelMarketIx(
  accounts: Accounts,
  { id, yesMint, noMint }: CancelMarketParams,
): Promise<Instruction> {
  const marketPda = await getMarketPda(id);

  const [briberAta] = await findAssociatedTokenPda({
    mint: constants.WSOL_MINT,
    owner: accounts.briber.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const [marketIncentiveVaultPda, marketYesVaultPda, marketNoVaultPda] = await Promise.all([
    getMarketVaultPda(marketPda, constants.WSOL_MINT),
    getMarketVaultPda(marketPda, yesMint),
    getMarketVaultPda(marketPda, noMint),
  ]);

  // Layout: [u8 discriminator=3, ...utf8 id]
  const ixData = Buffer.concat([Buffer.from([3]), Buffer.from(id, "utf8")]);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: accounts.briber.address, role: AccountRole.WRITABLE_SIGNER },
      { address: marketPda, role: AccountRole.WRITABLE },
      { address: constants.WSOL_MINT, role: AccountRole.READONLY },
      { address: briberAta, role: AccountRole.WRITABLE },
      { address: marketIncentiveVaultPda, role: AccountRole.WRITABLE },
      { address: marketYesVaultPda, role: AccountRole.WRITABLE },
      { address: marketNoVaultPda, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };
}
