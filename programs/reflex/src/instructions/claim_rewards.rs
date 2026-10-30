use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    mint, token,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, utils, Pool, Position};

#[derive(Accounts)]
pub struct ClaimRewards {
    #[account(mut)]
    pub staker: Signer,

    #[account(
        mut,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump = pool.bump,
    )]
    pub pool: Box<BorshAccount<Pool>>,

    /// CHECK: no validation because it could be closed
    // #[account(/* owner = *WORLD_PROGRAM_ID, */ constraint = market.data_len() == 0)]
    pub market: UncheckedAccount,

    #[account(
        mut,
        close = staker,
        seeds = [POSITION_SEED, pool.address().as_ref(), staker.address().as_ref()],
        bump = position.bump
    )]
    pub position: Box<BorshAccount<Position>>,

    #[account(address = pool.incentive_mint)]
    pub mint: Box<InterfaceAccount<Mint>>,
    #[account(mint::token_program = token_program)]
    pub outcome_mint: Box<InterfaceAccount<Mint>>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = pool,
        token::token_program = spl_token_program,
        seeds = [VAULT_SEED, pool.address().as_ref()],
        bump = pool.vault_bump
    )]
    pub pool_vault: Box<InterfaceAccount<TokenAccount>>,

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
        init_if_needed,
        payer = staker,
        associated_token::mint = mint,
        associated_token::authority = staker,
        associated_token::token_program = spl_token_program
    )]
    pub staker_ata: Box<InterfaceAccount<TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = outcome_mint,
        associated_token::authority = staker,
        associated_token::token_program = token_program,
    )]
    pub staker_outcome_ata: Box<InterfaceAccount<TokenAccount>>,

    pub associated_token_program: Program<AssociatedToken>,
    pub spl_token_program: Interface<TokenInterface>,
    pub token_program: Program<Token2022>,
    pub system_program: Program<System>,
}

impl ClaimRewards {
    pub fn proccess_ix(&mut self) -> Result<()> {
        require!(
            *self.outcome_mint.address() == self.pool.yes_outcome_mint
                || *self.outcome_mint.address() == self.pool.no_outcome_mint,
            ReflexError::WrongMint
        );

        let ts = self.pool.resolution_timestamp;

        // update pool and position
        let (stake_seconds, staked, s_w) = if self.pool.is_yes() {
            self.position.accrue_yes(ts)?;
            (
                self.position.yes_stake_seconds,
                self.position.yes_staked,
                self.pool.yes_stake_seconds,
            )
        } else if self.pool.is_no() {
            self.position.accrue_no(ts)?;
            (
                self.position.no_stake_seconds,
                self.position.no_staked,
                self.pool.no_stake_seconds,
            )
        } else {
            return Err(ReflexError::PooltNotSettled.into());
        };

        let payout: u64 = if s_w == 0 {
            0
        } else {
            utils::normalized_share(self.pool.total_incentive_amount, stake_seconds, s_w)?
        };

        require!(staked > 0 || payout > 0, ReflexError::ZeroRewards);

        // transfer outcome winning to staker
        let market_binding = self.market.address();
        let signer_seeds: &[&[&[u8]]] = &[&[POOL_SEED, market_binding.as_ref(), &[self.pool.bump]]];

        if staked > 0 {
            let cpi_accounts = token_interface::TransferChecked {
                from: self.pool_outcome_vault.to_cpi_handle_mut(),
                mint: self.outcome_mint.to_cpi_handle(),
                to: self.staker_outcome_ata.to_cpi_handle_mut(),
                authority: self.pool.to_cpi_handle(),
            };

            let cpi_ctx = CpiContext::new(self.token_program.address(), cpi_accounts)
                .with_signer(signer_seeds);

            token_interface::transfer_checked(cpi_ctx, staked, self.outcome_mint.decimals())?;
        }

        // transfer rewards to staker
        if payout > 0 {
            let cpi_accounts = token_interface::TransferChecked {
                from: self.pool_vault.to_cpi_handle_mut(),
                mint: self.mint.to_cpi_handle(),
                to: self.staker_ata.to_cpi_handle_mut(),
                authority: self.pool.to_cpi_handle(),
            };

            let cpi_ctx = CpiContext::new(self.spl_token_program.address(), cpi_accounts)
                .with_signer(signer_seeds);

            token_interface::transfer_checked(cpi_ctx, payout, self.mint.decimals())?;
        }
        Ok(())
    }
}
