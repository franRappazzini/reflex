use anchor_lang::prelude::*;

use crate::{constants::*, error::ReflexError, Config};

#[derive(Accounts)]
pub struct Initialize {
    #[account(mut)]
    pub authority: Signer,

    #[account(
        init,
        payer = authority,
        space = Config::SIZE,
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: BorshAccount<Config>,

    pub system_program: Program<System>,
}

impl Initialize {
    pub fn proccess_ix(
        &mut self,
        fee_bps: u16,
        briber_fee_bps: u16,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        require!(
            fee_bps < 5_000 && briber_fee_bps < 5_000,
            ReflexError::InvalidAmount
        );

        // self.config.set_inner(Config {
        //     authority: *self.authority.key,
        //     fee_bps,
        //     briber_fee_bps,
        //     bump: bumps.config,
        // });

        self.config.authority = *self.authority.address();
        self.config.fee_bps = fee_bps;
        self.config.briber_fee_bps = briber_fee_bps;
        self.config.bump = bumps.config;

        Ok(())
    }
}
