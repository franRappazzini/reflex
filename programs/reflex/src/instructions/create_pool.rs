use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, Config, Funder, Pool};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = signer,
        space = Pool::SIZE,
        seeds = [POOL_SEED, market.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = signer,
        space = Funder::SIZE,
        seeds = [FUNDER_SEED, pool.key().as_ref(), signer.key().as_ref()],
        bump
    )]
    pub funder: Account<'info, Funder>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount<'info>,

    #[account(
        mint::token_program = spl_token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    pub yes_outcome_mint: InterfaceAccount<'info, Mint>,
    pub no_outcome_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = pool,
        token::token_program = spl_token_program,
        seeds = [VAULT_SEED, pool.key().as_ref()],
        bump
    )]
    pub pool_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        token::mint = yes_outcome_mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.key().as_ref(), yes_outcome_mint.key().as_ref()],
        bump
    )]
    pub pool_yes_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        token::mint = no_outcome_mint,
        token::authority = pool,
        seeds = [VAULT_SEED, pool.key().as_ref(), no_outcome_mint.key().as_ref()],
        bump
    )]
    pub pool_no_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = spl_token_program
    )]
    pub signer_ata: InterfaceAccount<'info, TokenAccount>,

    pub spl_token_program: Interface<'info, TokenInterface>, // to usdc
    pub token_program: Program<'info, Token2022>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreatePool<'info> {
    pub fn proccess_ix(&mut self, amount: u64, bumps: CreatePoolBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);
        if !cfg!(feature = "local-test") {
            require!(
                MINT_WHITELIST.contains(&self.mint.key()),
                ReflexError::WrongMint
            );
        }

        // get world market data
        let market_ptr = self.market.try_borrow_data().unwrap().as_ptr();

        // TEST: to avoid real mint checks
        let (yes_outcome_mint, no_outcome_mint) = if !cfg!(feature = "local-test") {
            // SAFE: because market is validated by owner and data len
            (
                unsafe { *(market_ptr.add(WORLD_YES_MINT_OFFSET) as *const Pubkey) },
                unsafe { *(market_ptr.add(WORLD_NO_MINT_OFFSET) as *const Pubkey) },
            )
        } else {
            (self.yes_outcome_mint.key(), self.no_outcome_mint.key())
        };

        msg!("yes: {}", yes_outcome_mint);
        msg!("no: {}", no_outcome_mint);

        require!(
            yes_outcome_mint == self.yes_outcome_mint.key(),
            ReflexError::WrongMint
        );
        require!(
            no_outcome_mint == self.no_outcome_mint.key(),
            ReflexError::WrongMint
        );

        // transfer incentives to vault
        let cpi_accounts = token_interface::TransferChecked {
            authority: self.signer.to_account_info(),
            from: self.signer_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.pool_vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(*self.spl_token_program.key, cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        // set funder and pool account
        let now = Clock::get()?.unix_timestamp;

        self.funder.set_inner(Funder {
            weight_seconds: 0,
            amount,
            last_update_ts: now,
            funder: *self.signer.key,
            bump: bumps.funder,
        });

        self.pool.set_inner(Pool {
            creator: *self.signer.key,
            incentive_mint: self.mint.key(),
            yes_outcome_mint,
            no_outcome_mint,
            total_incentive_amount: amount,
            total_yes_staked: 0,
            total_no_staked: 0,
            available_yes_fees: 0,
            available_no_fees: 0,
            creation_timestamp: now,
            resolution_timestamp: 0,
            yes_last_update_ts: now,
            yes_stake_seconds: 0,
            no_last_update_ts: now,
            no_stake_seconds: 0,
            funder_last_update_ts: now,
            funder_weight_seconds: 0,
            fee_bps: self.config.fee_bps,
            status: crate::PoolStatus::Open,
            vault_bump: bumps.pool_vault,
            bump: bumps.pool,
        });

        Ok(())
    }
}
