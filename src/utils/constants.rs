use pinocchio::Address;

// addresses
pub const WSOL_ADDRESS: &Address =
    &Address::from_str_const("So11111111111111111111111111111111111111112");
pub const USDC_ADDRESS: &Address =
    &Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// seeds
pub const CONFIG_SEED: &[u8] = b"config";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const MARKET_SEED: &[u8] = b"market";
