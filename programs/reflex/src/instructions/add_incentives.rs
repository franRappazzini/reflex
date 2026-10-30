use anchor_lang::prelude::*;
use anchor_spl::{
    token,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, Funder, Pool};

#[derive(Accounts)]
pub struct AddIncentives {
    #[account(mut)]
    pub signer: Signer,

    #[account(
        mut,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump = pool.bump,
        constraint = pool.is_open() @ ReflexError::MarketIsNotOpen
    )]
    pub pool: BorshAccount<Pool>,

    #[account(
        init_if_needed,
        payer = signer,
        space = Funder::SIZE,
        seeds = [FUNDER_SEED, pool.address().as_ref(), signer.address().as_ref()],
        bump
    )]
    pub funder: BorshAccount<Funder>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount,

    #[account(address = pool.incentive_mint)]
    pub mint: InterfaceAccount<Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.address().as_ref()],
        bump = pool.vault_bump
    )]
    pub pool_vault: InterfaceAccount<TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub signer_ata: InterfaceAccount<TokenAccount>,

    pub token_program: Interface<TokenInterface>,
    pub system_program: Program<System>,
}

impl AddIncentives {
    pub fn proccess_ix(&mut self, amount: u64, bumps: &AddIncentivesBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);

        // transfer incentives to vault
        let cpi_accounts = token_interface::TransferChecked {
            from: self.signer_ata.to_cpi_handle_mut(),
            mint: self.mint.to_cpi_handle(),
            to: self.pool_vault.to_cpi_handle_mut(),
            authority: self.signer.to_cpi_handle(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.address(), cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals())?;

        // update funder and pool accounts
        let now = Clock::get()?.unix_timestamp;

        if self.funder.funder == Address::default() {
            self.funder.last_update_ts = now;
            self.funder.funder = *self.signer.address();
            self.funder.bump = bumps.funder;
        }

        self.pool.accrue_funders(now)?;
        self.funder.accrue(now)?;

        self.pool.total_incentive_amount = self
            .pool
            .total_incentive_amount
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        self.funder.amount = self
            .funder
            .amount
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }
}
