use anchor_lang::prelude::*;

use crate::{constants::*, error::ReflexError, Config, Pool, PoolStatus};

#[derive(Accounts)]
pub struct SettlePool<'info> {
    #[account(mut, address = config.authority)]
    pub authority: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump = pool.bump,
        constraint = pool.can_settle() @ ReflexError::CanNotSettle
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: no validation because it could be closed
    // #[account(/* owner = *predict_program.key, */ constraint = market.data_len() == 0)]
    pub market: UncheckedAccount<'info>,
}

impl<'info> SettlePool<'info> {
    // manual settlement until complete world integration
    pub fn proccess_ix(&mut self, decision: PoolStatus) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        self.pool.accrue_yes(now)?;
        self.pool.accrue_no(now)?;
        self.pool.resolution_timestamp = now;
        self.pool.status = decision;

        Ok(())
    }
}
