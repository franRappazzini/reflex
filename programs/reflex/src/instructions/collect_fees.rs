use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    mint, token,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, utils, Funder, Pool};

#[derive(Accounts)]
pub struct CollectFees {
    #[account(mut)]
    pub signer: Signer,

    #[account(
        mut,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump = pool.bump,
        constraint = pool.is_yes() || pool.is_no() @ ReflexError::PooltNotSettled
    )]
    pub pool: BorshAccount<Pool>,

    #[account(
        mut,
        close = signer,
        seeds = [FUNDER_SEED, pool.address().as_ref(), signer.address().as_ref()],
        bump = funder.bump
    )]
    pub funder: BorshAccount<Funder>,

    /// CHECK: no validation because it could be closed
    // #[account(/* owner = *predict_program.key, */ constraint = market.data_len() == 0)]
    pub market: UncheckedAccount,

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
        init_if_needed,
        payer = signer,
        associated_token::mint = outcome_mint,
        associated_token::authority = signer,
        associated_token::token_program = token_program
    )]
    pub signer_ata: InterfaceAccount<TokenAccount>,

    pub associated_token_program: Program<AssociatedToken>,
    pub token_program: Interface<TokenInterface>,
    pub system_program: Program<System>,
}

impl CollectFees {
    pub fn proccess_ix(&mut self) -> Result<()> {
        self.funder.accrue(self.pool.resolution_timestamp)?;

        let total_collected_fees = if self.pool.is_yes()
            && *self.outcome_mint.address() == self.pool.yes_outcome_mint
        {
            self.pool.available_yes_fees
        } else if self.pool.is_no() && *self.outcome_mint.address() == self.pool.no_outcome_mint {
            self.pool.available_no_fees
        } else {
            return Err(ReflexError::WrongMint.into());
        };

        let funder_collected_fee = utils::normalized_share(
            total_collected_fees,
            self.funder.weight_seconds,
            self.pool.funder_weight_seconds,
        )?;

        // update pool
        if self.pool.is_yes() {
            self.pool.available_yes_fees = self
                .pool
                .available_yes_fees
                .checked_sub(funder_collected_fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        } else if self.pool.is_no() {
            self.pool.available_no_fees = self
                .pool
                .available_no_fees
                .checked_sub(funder_collected_fee)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }
        self.pool.funder_weight_seconds = self
            .pool
            .funder_weight_seconds
            .checked_sub(self.funder.weight_seconds)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // transfer the collected outcome fees to the signer
        if funder_collected_fee > 0 {
            let market_binding = self.market.address();
            let signer_seeds: &[&[&[u8]]] =
                &[&[POOL_SEED, market_binding.as_ref(), &[self.pool.bump]]];

            let cpi_accounts = token_interface::TransferChecked {
                from: self.pool_outcome_vault.to_cpi_handle_mut(),
                mint: self.outcome_mint.to_cpi_handle(),
                to: self.signer_ata.to_cpi_handle_mut(),
                authority: self.pool.to_cpi_handle(),
            };

            let cpi_ctx = CpiContext::new(self.token_program.address(), cpi_accounts)
                .with_signer(signer_seeds);

            token_interface::transfer_checked(
                cpi_ctx,
                funder_collected_fee,
                self.outcome_mint.decimals(),
            )?;
        }

        Ok(())
    }
}
