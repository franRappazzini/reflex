use anchor_lang::prelude::*;
use anchor_spl::{
    mint, token,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};

use crate::{constants::*, error::ReflexError, Pool, Position};

#[derive(Accounts)]
pub struct UnstakeOutcome {
    #[account(mut)]
    pub staker: Signer,

    #[account(
        mut,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump = pool.bump,
        constraint = pool.is_open() @ ReflexError::MarketIsNotOpen
    )]
    pub pool: BorshAccount<Pool>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount,

    #[account(
        mut,
        seeds = [POSITION_SEED, pool.address().as_ref(), staker.address().as_ref()],
        bump
    )]
    pub position: BorshAccount<Position>,

    #[account(mint::token_program = token_program)]
    pub outcome_mint: InterfaceAccount<Mint>,

    #[account(
        mut,
        token::mint = outcome_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [VAULT_SEED, pool.address().as_ref(), outcome_mint.address().as_ref()],
        bump
    )]
    pub pool_outcome_vault: InterfaceAccount<TokenAccount>,

    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = staker,
        associated_token::token_program = token_program
    )]
    pub staker_ata: InterfaceAccount<TokenAccount>,

    pub token_program: Program<Token2022>,
}

impl UnstakeOutcome {
    pub fn proccess_ix(&mut self, amount: u64) -> Result<()> {
        require!(amount > 0, ReflexError::ZeroAmount);

        // current timestamp because the unstake is before the market resolution
        let now = Clock::get()?.unix_timestamp;

        let (pool_staked, position_staked) =
            if *self.outcome_mint.address() == self.pool.yes_outcome_mint {
                self.pool.accrue_yes(now)?;
                self.position.accrue_yes(now)?;

                (
                    &mut self.pool.total_yes_staked,
                    &mut self.position.yes_staked,
                )
            } else if *self.outcome_mint.address() == self.pool.no_outcome_mint {
                self.pool.accrue_no(now)?;
                self.position.accrue_no(now)?;

                (&mut self.pool.total_no_staked, &mut self.position.no_staked)
            } else {
                return Err(ReflexError::WrongMint.into());
            };

        require!(amount <= *position_staked, ReflexError::InvalidAmount);

        *pool_staked = pool_staked
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        *position_staked = position_staked
            .checked_sub(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // transfer outcome back to staker
        let market_binding = self.market.address();
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, market_binding.as_ref(), &[self.pool.bump]]];

        let cpi_accounts = token_interface::TransferChecked {
            authority: self.pool.to_cpi_handle(),
            from: self.pool_outcome_vault.to_cpi_handle_mut(),
            mint: self.outcome_mint.to_cpi_handle(),
            to: self.staker_ata.to_cpi_handle_mut(),
        };

        let cpi_ctx =
            CpiContext::new(self.token_program.address(), cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_ctx, amount, self.outcome_mint.decimals())
    }
}
