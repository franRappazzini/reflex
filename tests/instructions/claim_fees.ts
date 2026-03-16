import { AccountRole, Address, Instruction } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import { getMarketPda, getMarketVaultPda } from "../utils/pda";

import { Accounts } from "../utils/accounts";
import { constants } from "../utils/constants";

export type ClaimFeesParams = {
  id: string;
  /**
   * The resolved outcome mint — `yesMint` if the market resolved Yes,
   * `noMint` if it resolved No. Matches `market.outcome_yes_mint` or
   * `market.outcome_no_mint` read from on-chain state.
   */
  outcomeMint: Address;
};

/**
 * Builds the `claim_fees` instruction.
 *
 * Accounts (in order):
 *   briber (writable signer), market (writable), outcome_mint (readonly),
 *   briber_ata (writable), market_outcome_vault (writable), token_program
 *
 * Data layout: [u8 discriminator=4, ...utf8 id]
 *
 * Pre-conditions:
 *   - Market must be resolved (yes or no).
 *   - Market must have available fees > 0 on the resolved side.
 *   - The briber's ATA for `outcomeMint` must already exist.
 */
export async function buildClaimFeesIx(
  accounts: Accounts,
  { id, outcomeMint }: ClaimFeesParams,
): Promise<Instruction> {
  const marketPda = await getMarketPda(id);

  const [briberAta] = await findAssociatedTokenPda({
    mint: outcomeMint,
    owner: accounts.briber.address,
    tokenProgram: TOKEN_PROGRAM_ADDRESS,
  });

  const marketOutcomeVaultPda = await getMarketVaultPda(marketPda, outcomeMint);

  // Layout: [u8 discriminator=4, ...utf8 id]
  const ixData = Buffer.concat([
    Buffer.from([constants.CLAIM_FEES_DISCRIMINATOR]),
    Buffer.from(id, "utf8"),
  ]);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: accounts.briber.address, role: AccountRole.WRITABLE_SIGNER },
      { address: marketPda, role: AccountRole.WRITABLE },
      { address: outcomeMint, role: AccountRole.READONLY },
      { address: briberAta, role: AccountRole.WRITABLE },
      { address: marketOutcomeVaultPda, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };
}
