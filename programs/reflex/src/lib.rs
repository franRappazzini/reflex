pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("HQqk2aUfK95RFvwwQn3C1cP4cev18U9mWMW4nC35xJj1");

#[program]
pub mod reflex {
    use super::*;

    pub fn initialize(
        ctx: &mut Context<Initialize>,
        fee_bps: u16,
        briber_fee_bps: u16,
    ) -> Result<()> {
        ctx.accounts
            .proccess_ix(fee_bps, briber_fee_bps, &ctx.bumps)
    }

    pub fn create_pool(ctx: &mut Context<CreatePool>, amount: u64) -> Result<()> {
        ctx.accounts.proccess_ix(amount, &ctx.bumps)
    }

    pub fn add_incentives(ctx: &mut Context<AddIncentives>, amount: u64) -> Result<()> {
        ctx.accounts.proccess_ix(amount, &ctx.bumps)
    }

    pub fn stake_outcome(ctx: &mut Context<StakeOutcome>, amount: u64) -> Result<()> {
        ctx.accounts.proccess_ix(amount, &ctx.bumps)
    }

    pub fn unstake_outcome(ctx: &mut Context<UnstakeOutcome>, amount: u64) -> Result<()> {
        ctx.accounts.proccess_ix(amount)
    }

    pub fn settle_pool(ctx: &mut Context<SettlePool>, decision: PoolStatus) -> Result<()> {
        ctx.accounts.proccess_ix(decision)
    }

    pub fn claim_rewards(ctx: &mut Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.proccess_ix()
    }

    pub fn collect_fees(ctx: &mut Context<CollectFees>) -> Result<()> {
        ctx.accounts.proccess_ix()
    }
}
