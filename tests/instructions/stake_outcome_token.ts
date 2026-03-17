import { AccountRole, Address, Instruction, TransactionSigner } from "@solana/kit";
import { TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda } from "@solana-program/token";
import { getFarmerPositionPda, getMarketPda, getMarketVaultPda } from "../utils/pda";

import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { constants } from "../utils/constants";

interface StakeOutcomeTokenParams {
  id: string;
  amount: bigint;
  outcomeMint: Address;
  farmer: TransactionSigner;
}

/**
 * Builds the `stake_outcome_token` instruction.
 *
 * Accounts (in order):
 *   farmer (writable signer), market (writable), farmer_position PDA (writable),
 *   outcome_mint (readonly), farmer_ata (writable), market_outcome_vault (writable),
 *   token_program (readonly)
 *
 * Data layout: [u8 discriminator=7, u64 amount LE, ...utf8 market_id]
 */
export async function buildStakeOutcomeTokenIx(
  params: StakeOutcomeTokenParams,
): Promise<Instruction> {
  const { id, amount, outcomeMint, farmer } = params;

  const marketAddress = await getMarketPda(id);

  const [[farmerAta], farmerPositionAddress, marketOutcomeVault] = await Promise.all([
    findAssociatedTokenPda({
      mint: outcomeMint,
      owner: farmer.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    }),
    getFarmerPositionPda(marketAddress, farmer.address),
    getMarketVaultPda(marketAddress, outcomeMint),
  ]);

  // Data layout: [u8 disc=7, u64 amount LE, ...utf8 market_id]
  const idBytes = Buffer.from(id, "utf-8");
  const ixData = Buffer.alloc(1 + 8 + idBytes.length);
  ixData.writeUInt8(constants.STAKE_OUTCOME_TOKEN_DISCRIMINATOR, 0);
  ixData.writeBigUInt64LE(amount, 1);
  idBytes.copy(ixData, 9);

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: farmer.address, role: AccountRole.WRITABLE_SIGNER },
      { address: marketAddress, role: AccountRole.WRITABLE },
      { address: farmerPositionAddress, role: AccountRole.WRITABLE },
      { address: outcomeMint, role: AccountRole.READONLY },
      { address: farmerAta, role: AccountRole.WRITABLE },
      { address: marketOutcomeVault, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
      { address: SYSTEM_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };
}
