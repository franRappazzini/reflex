use anchor_lang::prelude::*;
use anchor_spl::{
    mint, token,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount},
};

use crate::{constants::*, error::ReflexError, utils, Pool, Position};

#[derive(Accounts)]
pub struct StakeOutcome {
    #[account(mut)]
    pub staker: Signer,

    #[account(
        mut,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump = pool.bump,
        constraint = pool.is_open() @ ReflexError::MarketIsNotOpen
    )]
    pub pool: Box<BorshAccount<Pool>>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount,

    #[account(
        init_if_needed,
        payer = staker,
        space = Position::SIZE,
        seeds = [POSITION_SEED, pool.address().as_ref(), staker.address().as_ref()],
        bump
    )]
    pub position: Box<BorshAccount<Position>>,

    #[account(mint::token_program = token_program)]
    pub outcome_mint: Box<InterfaceAccount<Mint>>,

    #[account(
        mut,
        token::mint = outcome_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [VAULT_SEED, pool.address().as_ref(), outcome_mint.address().as_ref()],
        bump
    )]
    pub pool_outcome_vault: Box<InterfaceAccount<TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = staker,
        associated_token::token_program = token_program,
    )]
    pub staker_ata: Box<InterfaceAccount<TokenAccount>>,

    pub token_program: Program<Token2022>,
    pub system_program: Program<System>,
}

impl StakeOutcome {
    pub fn proccess_ix(&mut self, amount: u64, bumps: &StakeOutcomeBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);

        require!(
            *self.outcome_mint.address() == self.pool.yes_outcome_mint
                || *self.outcome_mint.address() == self.pool.no_outcome_mint,
            ReflexError::WrongMint
        );

        // stake outcome tokens
        let cpi_accounts = token_interface::TransferChecked {
            authority: self.staker.to_cpi_handle(),
            from: self.staker_ata.to_cpi_handle_mut(),
            mint: self.outcome_mint.to_cpi_handle(),
            to: self.pool_outcome_vault.to_cpi_handle_mut(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.address(), cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.outcome_mint.decimals())?;

        // set if it's needed
        if self.position.authority == Address::default() {
            self.position.authority = *self.staker.address();
            self.position.bump = bumps.position;
        }

        let (amount_sub_fees, fees) = utils::apply_fee(amount, self.pool.fee_bps)?;

        let now = Clock::get()?.unix_timestamp;

        // update position and pool based on the outcome
        if *self.outcome_mint.address() == self.pool.yes_outcome_mint {
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
        } else if *self.outcome_mint.address() == self.pool.no_outcome_mint {
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
