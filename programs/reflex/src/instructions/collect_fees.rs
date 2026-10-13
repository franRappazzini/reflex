use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, Pool};

#[derive(Accounts)]
pub struct CollectFees<'info> {
    #[account(mut, address = pool.creator)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump = pool.bump,
        constraint = pool.is_yes() || pool.is_no() @ ReflexError::PooltNotSettled
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: no validation because it could be closed
    // #[account(/* owner = *predict_program.key, */ constraint = market.data_len() == 0)]
    pub market: UncheckedAccount<'info>,

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
        init_if_needed,
        payer = creator,
        associated_token::mint = outcome_mint,
        associated_token::authority = creator
    )]
    pub creator_ata: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CollectFees<'info> {
    pub fn proccess_ix(&mut self) -> Result<()> {
        let collected_fees =
            if self.pool.is_yes() && self.outcome_mint.key() == self.pool.yes_outcome_mint {
                let fees = self.pool.available_yes_fees;
                self.pool.available_yes_fees = 0;
                fees
            } else if self.pool.is_no() && self.outcome_mint.key() == self.pool.no_outcome_mint {
                let fees = self.pool.available_no_fees;
                self.pool.available_no_fees = 0;
                fees
            } else {
                return Err(ReflexError::WrongMint.into());
            };

        require!(collected_fees > 0, ReflexError::InvalidAmount);

        // Transfer the collected outcome fees to the creator.
        let market_binding = self.market.key();
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, market_binding.as_ref(), &[self.pool.bump]]];

        let cpi_accounts = token_interface::TransferChecked {
            from: self.pool_outcome_vault.to_account_info(),
            mint: self.outcome_mint.to_account_info(),
            to: self.creator_ata.to_account_info(),
            authority: self.pool.to_account_info(),
        };

        let cpi_ctx =
            CpiContext::new(*self.token_program.key, cpi_accounts).with_signer(signer_seeds);

        token_interface::transfer_checked(cpi_ctx, collected_fees, self.outcome_mint.decimals)
    }
}
