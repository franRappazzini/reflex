use anchor_lang::prelude::*;

use crate::DISCRIMINATOR_SIZE;

#[account(borsh)]
#[derive(InitSpace)]
pub struct Config {
    pub fee_bps: u16,
    pub briber_fee_bps: u16,
    pub authority: Address,
    pub bump: u8,
}

impl Config {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + Config::INIT_SPACE;
}
