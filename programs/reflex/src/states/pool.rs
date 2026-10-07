use anchor_lang::prelude::*;

use crate::DISCRIMINATOR_SIZE;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub creator: Pubkey,
    pub incentive_mint: Pubkey,
    pub yes_outcome_mint: Pubkey,
    pub no_outcome_mint: Pubkey,

    pub total_incentive_amount: u64,
    pub total_yes_staked: u64,
    pub total_no_staked: u64,

    pub available_yes_fees: u64,
    pub available_no_fees: u64,

    pub creation_timestamp: i64,
    pub update_timestamp: i64,

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
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone)]
pub enum PoolStatus {
    Open,
    Yes,
    No,
    Cancelled,
}
