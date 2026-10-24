use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};

use crate::{constants::*, error::ReflexError, utils, Pool, Position};

#[derive(Accounts)]
pub struct StakeOutcome<'info> {
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
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = staker,
        space = Position::SIZE,
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
        associated_token::authority = staker,
        associated_token::token_program = token_program,
    )]
    pub staker_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> StakeOutcome<'info> {
    pub fn proccess_ix(&mut self, amount: u64, bumps: StakeOutcomeBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);

        require!(
            self.outcome_mint.key() == self.pool.yes_outcome_mint
                || self.outcome_mint.key() == self.pool.no_outcome_mint,
            ReflexError::WrongMint
        );

        // stake outcome tokens
        let cpi_accounts = token_interface::TransferChecked {
            authority: self.staker.to_account_info(),
            from: self.staker_ata.to_account_info(),
            mint: self.outcome_mint.to_account_info(),
            to: self.pool_outcome_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(*self.token_program.key, cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.outcome_mint.decimals)?;

        // set if it's needed
        if self.position.authority == Pubkey::default() {
            self.position.authority = self.staker.key();
            self.position.bump = bumps.position;
        }

        let (amount_sub_fees, fees) = utils::apply_fee(amount, self.pool.fee_bps)?;

        let now = Clock::get()?.unix_timestamp;

        // update position and pool based on the outcome
        if self.outcome_mint.key() == self.pool.yes_outcome_mint {
            // fist accrue
            self.pool.accrue_yes(now)?;
            self.position.accrue_yes(now)?;

            // then update amounts
            self.pool.stake_yes(amount_sub_fees, fees)?;
            self.position.yes_staked = self
                .position
                .yes_staked
                .checked_add(amount_sub_fees)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        } else if self.outcome_mint.key() == self.pool.no_outcome_mint {
            // fist accrue
            self.pool.accrue_no(now)?;
            self.position.accrue_no(now)?;

            // then update amounts
            self.pool.stake_no(amount_sub_fees, fees)?;
            self.position.no_staked = self
                .position
                .no_staked
                .checked_add(amount_sub_fees)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        } else {
            return Err(ReflexError::WrongMint.into());
        }

        Ok(())
    }
}
