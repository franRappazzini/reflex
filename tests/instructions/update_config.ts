import { AccountRole, Address, Instruction, getAddressEncoder } from "@solana/kit";

import { Client } from "../utils/client";
import { constants } from "../utils/constants";
import { getConfigPda } from "../utils/pda";

export interface UpdateConfigParams {
  newAuthority: Address;
  newFeeBps: number;
  newBriberFeeBps: number;
}

/**
 * Builds the `update_config` instruction.
 *
 * Accounts (in order):
 *   authority (signer, writable), config (writable)
 *
 * Data layout (37 bytes):
 *   [u8  discriminator  ]  offset 0
 *   [u8; 32 new_authority] offset 1
 *   [u16 new_fee_bps LE ]  offset 33
 *   [u16 new_briber_fee_bps LE] offset 35
 */
export async function buildUpdateConfigIx(
  client: Client,
  params: UpdateConfigParams,
): Promise<Instruction> {
  const { newAuthority, newFeeBps, newBriberFeeBps } = params;

  const data = Buffer.alloc(37);
  data.writeUInt8(constants.UPDATE_CONFIG_DISCRIMINATOR, 0);
  data.set(getAddressEncoder().encode(newAuthority), 1);
  data.writeUInt16LE(newFeeBps, 33);
  data.writeUInt16LE(newBriberFeeBps, 35);

  const configPda = await getConfigPda();

  return {
    programAddress: constants.PROGRAM_ID,
    accounts: [
      { address: client.wallet.address, role: AccountRole.WRITABLE_SIGNER },
      { address: configPda, role: AccountRole.WRITABLE },
    ],
    data,
  };
}
