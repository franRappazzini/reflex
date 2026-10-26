use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::ReflexError, Funder, Pool};

#[derive(Accounts)]
pub struct AddIncentives<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump = pool.bump,
        constraint = pool.is_open() @ ReflexError::MarketIsNotOpen
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = Funder::SIZE,
        seeds = [FUNDER_SEED, pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub funder: Box<Account<'info, Funder>>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount<'info>,

    #[account(address = pool.incentive_mint)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump
    )]
    pub pool_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub signer_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddIncentives<'info> {
    pub fn proccess_ix(&mut self, amount: u64, bumps: AddIncentivesBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);

        // transfer incentives to vault
        let cpi_accounts = token_interface::TransferChecked {
            from: self.signer_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.pool_vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(*self.token_program.key, cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        // update funder and pool accounts
        let now = Clock::get()?.unix_timestamp;

        if self.funder.funder == Pubkey::default() {
            self.funder.set_inner(Funder {
                weight_seconds: 0,
                amount: 0,
                last_update_ts: now,
                funder: *self.signer.key,
                bump: bumps.funder,
            });
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
