import {
  Address,
  Rpc,
  SolanaRpcApi,
  fetchEncodedAccount,
  getStructCodec,
  getU64Codec,
  getU8Codec,
} from "@solana/kit";

// Mirrors src/states/farmer_position.rs — FarmerPosition #[repr(C)]
// Layout (18 bytes):
//   [u8;  8]  yes_staked  (u64 LE)
//   [u8;  8]  no_staked   (u64 LE)
//   [u8;  1]  is_initialized
//   [u8;  1]  bump
const farmerPositionCodec = getStructCodec([
  ["yesStaked", getU64Codec()],
  ["noStaked", getU64Codec()],
  ["isInitialized", getU8Codec()],
  ["bump", getU8Codec()],
]);

export type FarmerPositionAccount = ReturnType<typeof farmerPositionCodec.decode>;

/**
 * Returns the decoded FarmerPosition account, or `null` if the account does not exist.
 * Use this when validating that a position was closed.
 */
export async function fetchMaybeFarmerPosition(
  rpc: Rpc<SolanaRpcApi>,
  address: Address,
): Promise<FarmerPositionAccount | null> {
  const account = await fetchEncodedAccount(rpc, address);
  if (!account.exists) return null;
  return farmerPositionCodec.decode(account.data);
}

/** Returns the decoded FarmerPosition account. Throws if the account does not exist. */
export async function fetchFarmerPosition(
  rpc: Rpc<SolanaRpcApi>,
  address: Address,
): Promise<FarmerPositionAccount> {
  const position = await fetchMaybeFarmerPosition(rpc, address);
  if (!position) throw new Error(`FarmerPosition account not found: ${address}`);
  return position;
}
