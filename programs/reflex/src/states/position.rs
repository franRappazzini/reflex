use anchor_lang::prelude::*;

use crate::{utils, DISCRIMINATOR_SIZE};

#[account(borsh)]
#[derive(InitSpace)]
pub struct Position {
    pub yes_stake_seconds: u128,
    pub no_stake_seconds: u128,
    pub yes_staked: u64,
    pub no_staked: u64,
    pub yes_last_update_ts: i64,
    pub no_last_update_ts: i64,
    pub authority: Address,
    pub bump: u8,
}

impl Position {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + Position::INIT_SPACE;

    pub fn accrue_yes(&mut self, ts: i64) -> Result<()> {
        utils::accrue(
            self.yes_staked,
            &mut self.yes_stake_seconds,
            &mut self.yes_last_update_ts,
            ts,
        )
    }

    pub fn accrue_no(&mut self, ts: i64) -> Result<()> {
        utils::accrue(
            self.no_staked,
            &mut self.no_stake_seconds,
            &mut self.no_last_update_ts,
            ts,
        )
    }
}
