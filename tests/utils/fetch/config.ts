import {
  Address,
  Rpc,
  SolanaRpcApi,
  assertAccountExists,
  fetchEncodedAccount,
  getAddressCodec,
  getStructCodec,
  getU16Codec,
  getU8Codec,
} from "@solana/kit";

// Mirrors src/states/config.rs — Config #[repr(C)]
// Layout (37 bytes):
//   [u8; 32]  authority
//   [u8;  2]  fee_bps       (u16 LE)
//   [u8;  2]  briber_fee_bps (u16 LE)
//   [u8;  1]  bump
const configCodec = getStructCodec([
  ["authority", getAddressCodec()],
  ["feeBps", getU16Codec()],
  ["briberFeeBps", getU16Codec()],
  ["bump", getU8Codec()],
]);

export type ConfigAccount = {
  authority: Address;
  feeBps: number;
  briberFeeBps: number;
  bump: number;
};

export async function fetchConfig(
  rpc: Rpc<SolanaRpcApi>,
  address: Address,
): Promise<ConfigAccount> {
  const account = await fetchEncodedAccount(rpc, address);
  assertAccountExists(account);
  return configCodec.decode(account.data);
}
