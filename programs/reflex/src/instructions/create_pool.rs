use anchor_lang::prelude::*;
use anchor_spl::{
    mint, token,
    token_2022::Token2022,
    token_interface::{self, Mint, TokenAccount, TokenInterface},
};

use crate::{constants::*, error::ReflexError, Config, Funder, Pool};

#[derive(Accounts)]
pub struct CreatePool {
    #[account(mut)]
    pub signer: Signer,

    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Box<BorshAccount<Config>>,

    #[account(
        init,
        payer = signer,
        space = Pool::SIZE,
        seeds = [POOL_SEED, market.address().as_ref()],
        bump
    )]
    pub pool: Box<BorshAccount<Pool>>,

    #[account(
        init,
        payer = signer,
        space = Funder::SIZE,
        seeds = [FUNDER_SEED, pool.address().as_ref(), signer.address().as_ref()],
        bump
    )]
    pub funder: Box<BorshAccount<Funder>>,

    /// CHECK: by owner and data len
    #[account(owner = WORLD_PROGRAM_ID, constraint = market.data_len() == WORLD_MARKET_SIZE)]
    pub market: UncheckedAccount,

    #[account(
        mint::token_program = spl_token_program,
    )]
    pub mint: Box<InterfaceAccount<Mint>>,

    #[account(mint::token_program = token_program)]
    pub yes_outcome_mint: Box<InterfaceAccount<Mint>>,

    #[account(mint::token_program = token_program)]
    pub no_outcome_mint: Box<InterfaceAccount<Mint>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = pool,
        token::token_program = spl_token_program,
        seeds = [VAULT_SEED, pool.address().as_ref()],
        bump
    )]
    pub pool_vault: Box<InterfaceAccount<TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = yes_outcome_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [VAULT_SEED, pool.address().as_ref(), yes_outcome_mint.address().as_ref()],
        bump
    )]
    pub pool_yes_vault: Box<InterfaceAccount<TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = no_outcome_mint,
        token::authority = pool,
        token::token_program = token_program,
        seeds = [VAULT_SEED, pool.address().as_ref(), no_outcome_mint.address().as_ref()],
        bump
    )]
    pub pool_no_vault: Box<InterfaceAccount<TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = signer,
        associated_token::token_program = spl_token_program
    )]
    pub signer_ata: Box<InterfaceAccount<TokenAccount>>,

    pub spl_token_program: Interface<TokenInterface>, // to usdc
    pub token_program: Program<Token2022>,
    pub system_program: Program<System>,
}

impl CreatePool {
    pub fn proccess_ix(&mut self, amount: u64, bumps: &CreatePoolBumps) -> Result<()> {
        require!(amount > 0, ReflexError::InvalidAmount);
        require!(
            MINT_WHITELIST.contains(&self.mint.address()),
            ReflexError::WrongMint
        );

        let (yes_outcome_mint, no_outcome_mint) = {
            let market_data = self.market.try_borrow()?;
            let yes_outcome_mint = Address::new_from_array(
                market_data[WORLD_YES_MINT_OFFSET..WORLD_YES_MINT_OFFSET + 32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            );
            let no_outcome_mint = Address::new_from_array(
                market_data[WORLD_NO_MINT_OFFSET..WORLD_NO_MINT_OFFSET + 32]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidAccountData)?,
            );
            (yes_outcome_mint, no_outcome_mint)
        };

        msg!("yes: {}", yes_outcome_mint);
        msg!("no: {}", no_outcome_mint);

        require!(
            yes_outcome_mint == *self.yes_outcome_mint.address(),
            ReflexError::WrongMint
        );
        require!(
            no_outcome_mint == *self.no_outcome_mint.address(),
            ReflexError::WrongMint
        );

        // transfer incentives to vault
        let cpi_accounts = token_interface::TransferChecked {
            authority: self.signer.to_cpi_handle(),
            from: self.signer_ata.to_cpi_handle_mut(),
            mint: self.mint.to_cpi_handle(),
            to: self.pool_vault.to_cpi_handle_mut(),
        };

        let cpi_ctx = CpiContext::new(self.spl_token_program.address(), cpi_accounts);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals())?;

        // set funder and pool account
        let now = Clock::get()?.unix_timestamp;

        // self.funder.set_inner(Funder {
        //     weight_seconds: 0,
        //     amount,
        //     last_update_ts: now,
        //     funder: *self.signer.key,
        //     bump: bumps.funder,
        // });

        self.funder.amount = amount;
        self.funder.last_update_ts = now;
        self.funder.funder = *self.signer.address();
        self.funder.bump = bumps.funder;

        // self.pool.set_inner(Pool {
        //     creator: *self.signer.key,
        //     incentive_mint: self.mint.address(),
        //     yes_outcome_mint,
        //     no_outcome_mint,
        //     total_incentive_amount: amount,
        //     total_yes_staked: 0,
        //     total_no_staked: 0,
        //     available_yes_fees: 0,
        //     available_no_fees: 0,
        //     creation_timestamp: now,
        //     resolution_timestamp: 0,
        //     yes_last_update_ts: now,
        //     yes_stake_seconds: 0,
        //     no_last_update_ts: now,
        //     no_stake_seconds: 0,
        //     funder_last_update_ts: now,
        //     funder_weight_seconds: 0,
        //     fee_bps: self.config.fee_bps,
        //     status: crate::PoolStatus::Open,
        //     vault_bump: bumps.pool_vault,
        //     bump: bumps.pool,
        // });

        self.pool.creator = *self.signer.address();
        self.pool.incentive_mint = *self.mint.address();
        self.pool.yes_outcome_mint = yes_outcome_mint;
        self.pool.no_outcome_mint = no_outcome_mint;
        self.pool.total_incentive_amount = amount;
        self.pool.creation_timestamp = now;
        self.pool.yes_last_update_ts = now;
        self.pool.no_last_update_ts = now;
        self.pool.funder_last_update_ts = now;
        self.pool.fee_bps = self.config.fee_bps;
        self.pool.status = crate::PoolStatus::Open;
        self.pool.vault_bump = bumps.pool_vault;
        self.pool.bump = bumps.pool;

        Ok(())
    }
}
