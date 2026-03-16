import { AccountRole, Instruction } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import { getConfigPda, getMarketPda, getMarketVaultPda, getTreasuryPda } from "../utils/pda";

import { Accounts } from "../utils/accounts";
import { constants } from "../utils/constants";

export type AddIncentivesParams = {
  id: string;
  /** Amount of incentive tokens to add (in lamports). */
  amount: bigint;
};

/**
 * Builds the `add_incentives` instruction.
 *
 * Accounts (in order):
 *   briber (writable signer), config, treasury,
 *   market, incentive_mint, briber_ata, market_incentive_vault
 *
 * Data layout (after discriminator stripped by the router):
 *   [u8 discriminator=2, u64 amount (LE), ...utf8 id]
 */
export async function buildAddIncentivesIx(
  accounts: Accounts,
  { id, amount }: AddIncentivesParams,
): Promise<Instruction> {
  const marketPda = await getMarketPda(id);

  const [briberAta] = await findAssociatedTokenPda({
    mint: constants.WSOL_MINT,
    owner: accounts.briber.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const [configPda, wsolTreasuryPda, marketIncentiveVaultPda] = await Promise.all([
    getConfigPda(),
    getTreasuryPda(constants.WSOL_MINT),
    getMarketVaultPda(marketPda, constants.WSOL_MINT),
  ]);

  // Layout: [u8 discriminator=2, u64 amount (LE), ...utf8 id]
  const header = Buffer.alloc(9);
  header.writeUInt8(constants.ADD_INCENTIVES_DISCRIMINATOR, 0);
  header.writeBigUInt64LE(amount, 1);
  const ixData = Buffer.concat([header, Buffer.from(id, "utf8")]);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: accounts.briber.address, role: AccountRole.WRITABLE_SIGNER },
      { address: configPda, role: AccountRole.READONLY },
      { address: wsolTreasuryPda, role: AccountRole.WRITABLE },
      { address: marketPda, role: AccountRole.WRITABLE },
      { address: constants.WSOL_MINT, role: AccountRole.READONLY },
      { address: briberAta, role: AccountRole.WRITABLE },
      { address: marketIncentiveVaultPda, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };
}
