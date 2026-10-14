use anchor_lang::prelude::*;

use crate::{utils, DISCRIMINATOR_SIZE};

#[account]
#[derive(InitSpace)]
pub struct Funder {
    pub weight_seconds: u128,
    pub amount: u64,
    pub last_update_ts: i64,
    pub funder: Pubkey,
    pub bump: u8,
}

impl Funder {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + Funder::INIT_SPACE;

    pub fn accrue(&mut self, ts: i64) -> Result<()> {
        utils::accrue(
            self.amount,
            &mut self.weight_seconds,
            &mut self.last_update_ts,
            ts,
        )
    }
}
