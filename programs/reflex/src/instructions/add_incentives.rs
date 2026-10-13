use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface};

use crate::{constants::*, error::ReflexError, Pool};

#[derive(Accounts)]
pub struct AddIncentives<'info> {
    // for now, the creator just can incentive the pool
    #[account(mut, address = pool.creator)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount<'info>,

    #[account(address = pool.incentive_mint)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump = pool.vault_bump
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub signer_ata: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> AddIncentives<'info> {
    pub fn proccess_ix(&mut self, amount: u64) -> Result<()> {
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

        // update pool
        self.pool.total_incentive_amount = self
            .pool
            .total_incentive_amount
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        Ok(())
    }
}
