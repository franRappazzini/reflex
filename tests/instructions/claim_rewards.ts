import { AccountRole, Address, Instruction, TransactionSigner } from "@solana/kit";
import {
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getCreateAssociatedTokenInstructionAsync,
} from "@solana-program/token";
import { getFarmerPositionPda, getMarketPda, getMarketVaultPda } from "../utils/pda";

import { constants } from "../utils/constants";

export interface ClaimRewardsParams {
  id: string;
  /** market.incentive_mint — the reward token (WSOL in tests). */
  rewardMint: Address;
  /** The winning outcome mint — yes or no based on market resolution. */
  outcomeMint: Address;
  farmer: TransactionSigner;
}

/**
 * Builds the instructions required for the `claim_rewards` flow:
 *   1. Create farmer's ATA for rewardMint if it doesn't exist yet.
 *   2. claim_rewards program instruction.
 *
 * Accounts (in order for the program ix):
 *   farmer (writable signer), market (writable), farmer_position PDA (writable),
 *   reward_mint (readonly), outcome_mint (readonly),
 *   market_reward_vault (writable), market_outcome_vault (writable),
 *   farmer_reward_ata (writable), farmer_outcome_ata (writable),
 *   token_program (readonly)
 *
 * Data layout: [u8 discriminator=9, ...utf8 market_id]
 *
 * Pre-conditions:
 *   - Market must be settled.
 *   - farmer_position must exist and the farmer must have staked on the winning side.
 *   - farmer_outcome_ata must already exist (created during stake).
 *
 * Post-conditions:
 *   - Reward tokens transferred from market_reward_vault → farmer_reward_ata.
 *   - Staked outcome tokens transferred from market_outcome_vault → farmer_outcome_ata.
 *   - farmer_position account is closed (rent returned to farmer).
 */
export async function buildClaimRewardsIxs(params: ClaimRewardsParams): Promise<Instruction[]> {
  const { id, rewardMint, outcomeMint, farmer } = params;

  const marketAddress = await getMarketPda(id);

  const [
    [farmerRewardAta],
    [farmerOutcomeAta],
    farmerPositionAddress,
    marketRewardVault,
    marketOutcomeVault,
  ] = await Promise.all([
    findAssociatedTokenPda({
      mint: rewardMint,
      owner: farmer.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    }),
    findAssociatedTokenPda({
      mint: outcomeMint,
      owner: farmer.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    }),
    getFarmerPositionPda(marketAddress, farmer.address),
    getMarketVaultPda(marketAddress, rewardMint),
    getMarketVaultPda(marketAddress, outcomeMint),
  ]);

  const createFarmerRewardAtaIx = await getCreateAssociatedTokenInstructionAsync({
    payer: farmer,
    ata: farmerRewardAta,
    owner: farmer.address,
    mint: rewardMint,
  });

  // Data layout: [u8 disc=9, ...utf8 market_id]
  const ixData = Buffer.concat([
    Buffer.from([constants.CLAIM_REWARDS_DISCRIMINATOR]),
    Buffer.from(id, "utf-8"),
  ]);

  const claimIx: Instruction = {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: farmer.address, role: AccountRole.WRITABLE_SIGNER },
      { address: marketAddress, role: AccountRole.WRITABLE },
      { address: farmerPositionAddress, role: AccountRole.WRITABLE },
      { address: rewardMint, role: AccountRole.READONLY },
      { address: outcomeMint, role: AccountRole.READONLY },
      { address: marketRewardVault, role: AccountRole.WRITABLE },
      { address: marketOutcomeVault, role: AccountRole.WRITABLE },
      { address: farmerRewardAta, role: AccountRole.WRITABLE },
      { address: farmerOutcomeAta, role: AccountRole.WRITABLE },
      { address: TOKEN_PROGRAM_ADDRESS, role: AccountRole.READONLY },
    ],
    data: ixData,
  };

  return [createFarmerRewardAtaIx, claimIx];
}
