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

    /// CHECK: by owner and data len
    // #[cfg(not(feature = "skip-checks"))] // in test we ignore if the real market exists
    // #[account(/* owner = *predict_program.key, */ constraint = market.data_len() == 0)]
    // TODO:
    pub market: UncheckedAccount<'info>,
}

impl<'info> SettlePool<'info> {
    pub fn proccess_ix(&mut self, decision: PoolStatus) -> Result<()> {
        self.pool.status = decision;

        Ok(())
    }
}
