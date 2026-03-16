import {
  Address,
  Rpc,
  SolanaRpcApi,
  assertAccountExists,
  fetchEncodedAccount,
  getAddressCodec,
  getI64Codec,
  getStructCodec,
  getU16Codec,
  getU64Codec,
  getU8Codec,
} from "@solana/kit";

// Mirrors src/states/market.rs — MarketStatus #[repr(u8)]
export enum MarketStatus {
  Open = 0,
  Settled = 1,
}

// Mirrors src/states/market.rs — MarketResolution #[repr(u8)]
export enum MarketResolution {
  None = 0,
  Yes = 1,
  No = 2,
}

// Mirrors src/states/market.rs — Market #[repr(C)]
// Layout (181 bytes):
//   [u8; 32]  briber
//   [u8; 32]  incentive_mint
//   [u8; 32]  outcome_yes_mint
//   [u8; 32]  outcome_no_mint
//   [u8;  8]  total_incentive_amount (u64 LE)
//   [u8;  8]  total_yes_staked       (u64 LE)
//   [u8;  8]  total_no_staked        (u64 LE)
//   [u8;  8]  available_yes_fees     (u64 LE)
//   [u8;  8]  available_no_fees      (u64 LE)
//   [u8;  8]  creation_timestamp     (i64 LE)
//   [u8;  2]  fee_bps                (u16 LE)
//   [u8;  1]  status                 (u8)
//   [u8;  1]  resolution             (u8)
//   [u8;  1]  bump
const marketCodec = getStructCodec([
  ["briber", getAddressCodec()],
  ["incentiveMint", getAddressCodec()],
  ["outcomeYesMint", getAddressCodec()],
  ["outcomeNoMint", getAddressCodec()],
  ["totalIncentiveAmount", getU64Codec()],
  ["totalYesStaked", getU64Codec()],
  ["totalNoStaked", getU64Codec()],
  ["availableYesFees", getU64Codec()],
  ["availableNoFees", getU64Codec()],
  ["creationTimestamp", getI64Codec()],
  ["feeBps", getU16Codec()],
  ["status", getU8Codec()],
  ["resolution", getU8Codec()],
  ["bump", getU8Codec()],
]);

export type MarketAccount = {
  briber: Address;
  incentiveMint: Address;
  outcomeYesMint: Address;
  outcomeNoMint: Address;
  totalIncentiveAmount: bigint;
  totalYesStaked: bigint;
  totalNoStaked: bigint;
  availableYesFees: bigint;
  availableNoFees: bigint;
  creationTimestamp: bigint;
  feeBps: number;
  status: MarketStatus;
  resolution: MarketResolution;
  bump: number;
};

export async function fetchMarket(
  rpc: Rpc<SolanaRpcApi>,
  address: Address,
): Promise<MarketAccount> {
  const account = await fetchEncodedAccount(rpc, address);
  assertAccountExists(account);
  const decoded = marketCodec.decode(account.data);
  return {
    ...decoded,
    status: decoded.status as MarketStatus,
    resolution: decoded.resolution as MarketResolution,
  };
}
