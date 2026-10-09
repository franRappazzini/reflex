use anchor_lang::prelude::*;

use crate::{utils, DISCRIMINATOR_SIZE};

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub creator: Pubkey,
    pub incentive_mint: Pubkey,
    pub yes_outcome_mint: Pubkey,
    pub no_outcome_mint: Pubkey,

    pub yes_stake_seconds: u128, // global accumulator
    pub no_stake_seconds: u128,  // global accumulator

    pub total_incentive_amount: u64,
    pub total_yes_staked: u64,
    pub total_no_staked: u64,

    pub available_yes_fees: u64,
    pub available_no_fees: u64,

    pub creation_timestamp: i64,
    pub resolution_timestamp: i64,
    pub yes_last_update_ts: i64,
    pub no_last_update_ts: i64,

    pub fee_bps: u16,
    pub status: PoolStatus,
    pub vault_bump: u8,
    pub bump: u8,
    // pub mint: Pubkey,
    // pub receipt_mint: Pubkey,
    // pub total_deposited: u64,
    // pub total_claimed: u64,
    // pub collected_fees: u64,
    // pub fees_claimed: bool,
}

impl Pool {
    pub const SIZE: usize = DISCRIMINATOR_SIZE + Pool::INIT_SPACE;

    pub fn accrue_yes(&mut self, ts: i64) -> Result<()> {
        utils::accrue(
            self.total_yes_staked,
            &mut self.yes_stake_seconds,
            &mut self.yes_last_update_ts,
            ts,
        )
    }

    pub fn accrue_no(&mut self, ts: i64) -> Result<()> {
        utils::accrue(
            self.total_no_staked,
            &mut self.no_stake_seconds,
            &mut self.no_last_update_ts,
            ts,
        )
    }

    pub fn stake_yes(&mut self, staked: u64, fee: u64) -> Result<()> {
        self.total_yes_staked = self
            .total_yes_staked
            .checked_add(staked)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.available_yes_fees = self
            .available_yes_fees
            .checked_add(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }

    pub fn stake_no(&mut self, staked: u64, fee: u64) -> Result<()> {
        self.total_no_staked = self
            .total_no_staked
            .checked_add(staked)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        self.available_no_fees = self
            .available_no_fees
            .checked_add(fee)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }

    pub fn effective_ts(&self, now: i64) -> i64 {
        if self.is_yes() || self.is_no() {
            self.resolution_timestamp
        } else {
            now
        }
    }

    pub fn is_open(&self) -> bool {
        self.status == PoolStatus::Open
    }

    pub fn is_yes(&self) -> bool {
        self.status == PoolStatus::Yes
    }

    pub fn is_no(&self) -> bool {
        self.status == PoolStatus::No
    }

    pub fn can_settle(&self) -> bool {
        self.is_open() || self.status == PoolStatus::Resolving
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, PartialEq)]
pub enum PoolStatus {
    Open,
    Yes,
    No,
    Resolving,
    Cancelled,
}
