use anchor_lang::{address, constant, Address};

pub const WORLD_PROGRAM_ID: Address = address!("prediCtPZCttYMvm2W3PtxmMxLmT1dtN7riU6Cxh6tM"); // mainnet
pub const USDC_MINT_ID: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // mainnet
pub const WSOL_MINT_ID: Address = address!("So11111111111111111111111111111111111111112"); // mainnet

pub const MINT_WHITELIST: [Address; 2] = [USDC_MINT_ID, WSOL_MINT_ID];

pub const DISCRIMINATOR_SIZE: usize = 8;
pub const WORLD_MARKET_SIZE: usize = 320;
pub const WORLD_YES_MINT_OFFSET: usize = 72;
pub const WORLD_NO_MINT_OFFSET: usize = 104;
pub const BPS_DENOMINATOR: u16 = 10_000;

// seeds
#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const POOL_SEED: &[u8] = b"pool";

#[constant]
pub const VAULT_SEED: &[u8] = b"vault";

#[constant]
pub const POSITION_SEED: &[u8] = b"position";

#[constant]
pub const FUNDER_SEED: &[u8] = b"funder";
