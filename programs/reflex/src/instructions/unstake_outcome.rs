use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::ReflexError, Pool, Position};

#[derive(Accounts)]
pub struct UnstakeOutcome<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump = pool.bump,
        constraint = pool.is_open() @ ReflexError::MarketIsNotOpen
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: by owner and data len
    #[account(owner = *predict_program.key, constraint = market.data_len() == 320)]
    pub market: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [POSITION_SEED, pool.key().as_ref(), staker.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    pub outcome_mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = outcome_mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.key().as_ref(), outcome_mint.key().as_ref()],
        bump
    )]
    pub pool_outcome_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = staker
    )]
    pub staker_ata: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: by id
    // #[account(address = Pubkey::from_str(""))]
    pub predict_program: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> UnstakeOutcome<'info> {
    pub fn proccess_ix(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, ReflexError::ZeroAmount);

        // current timestamp because the unstake is before the market resolution
        let now = Clock::get()?.unix_timestamp;

        let (pool_staked, position_staked) =
            if self.outcome_mint.key() == self.pool.yes_outcome_mint {
                self.pool.accrue_yes(now)?;
                self.position.accrue_yes(now)?;

                (
                    &mut self.pool.total_yes_staked,
                    &mut self.position.yes_staked,
                )
            } else if self.outcome_mint.key() == self.pool.no_outcome_mint {
                self.pool.accrue_no(now)?;
                self.position.accrue_no(now)?;

                (&mut self.pool.total_no_staked, &mut self.position.no_staked)
            } else {
                return Err(ReflexError::WrongMint.into());
            };

        require!(amount <= *position_staked, ReflexError::InvalidAmount);

        *pool_staked = pool_staked
            .checked_sub(amount)
            .ok_or(ReflexError::MathOverflow)?;
        *position_staked = position_staked
            .checked_sub(amount)
            .ok_or(ReflexError::MathOverflow)?;

        // transfer outcome back to staker
        let market_binding = self.market.key();
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, market_binding.as_ref(), &[self.pool.bump]]];

        let cpi_accounts = token_interface::TransferChecked {
            authority: self.pool.to_account_info(),
            from: self.pool_outcome_vault.to_account_info(),
            mint: self.outcome_mint.to_account_info(),
            to: self.staker_ata.to_account_info(),
        };

        let cpi_ctx =
            CpiContext::new(*self.token_program.key, cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_ctx, amount, self.outcome_mint.decimals)
    }
}
