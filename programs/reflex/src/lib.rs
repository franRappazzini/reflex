pub mod constants;
pub mod error;
pub mod instructions;
pub mod states;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use states::*;

declare_id!("8i3X3C7WaRGxKLTpoecUqmPEueMULXVTRzEJFuyGrTHd");

#[program]
pub mod reflex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, fee_bps: u16, briber_fee_bps: u16) -> Result<()> {
        ctx.accounts.proccess_ix(fee_bps, briber_fee_bps, ctx.bumps)
    }

    pub fn create_pool(ctx: Context<CreatePool>, amount: u64) -> Result<()> {
        ctx.accounts.proccess_ix(amount, ctx.bumps)
    }
}
