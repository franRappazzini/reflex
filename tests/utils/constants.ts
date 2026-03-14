import { address } from "@solana/kit";

// addresses
const PROGRAM_ID = address("4ZegtDo8WG6e2PAswLhnGXYDS5TGkniVCKXDrDX12KYX");
const WSOL_MINT = address("So11111111111111111111111111111111111111112");
const USDC_MINT = address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// seeds
const CONFIG_SEED = Buffer.from("config");
const TREASURY_SEED = Buffer.from("treasury");
const MARKET_SEED = Buffer.from("market");

export const constants = {
  PROGRAM_ID,
  WSOL_MINT,
  USDC_MINT,
  CONFIG_SEED,
  TREASURY_SEED,
  MARKET_SEED,
};
