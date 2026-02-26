#![cfg_attr(not(test), no_std)]

#[cfg(test)]
mod tests;

mod constants;
mod errors;
mod instructions;
mod states;
mod utils;

use pinocchio::{
    error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView,
    Address, ProgramResult,
};

use crate::instructions::{
    AddIncentives, CancelMarket, CreateMarketVault, Initialize, SettleMarket, StakeOutcomeToken,
    UnstakeOutcomeToken,
};

nostd_panic_handler!();
no_allocator!();
program_entrypoint!(process_entrypoint);

pinocchio::address::declare_id!("7zogcJaEsucGbcnZz4o4ARRbeF8AUU1RUP7zsAJ68wK7");

fn process_entrypoint(
    _program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    match data.split_first() {
        Some((Initialize::DISCRIMINATOR, data)) => {
            Initialize::try_from((data, accounts))?.process()
        }
        Some((CreateMarketVault::DISCRIMINATOR, data)) => {
            CreateMarketVault::try_from((data, accounts))?.process()
        }
        Some((StakeOutcomeToken::DISCRIMINATOR, data)) => {
            StakeOutcomeToken::try_from((data, accounts))?.process()
        }
        Some((UnstakeOutcomeToken::DISCRIMINATOR, data)) => {
            UnstakeOutcomeToken::try_from((data, accounts))?.process()
        }
        Some((AddIncentives::DISCRIMINATOR, data)) => {
            AddIncentives::try_from((data, accounts))?.process()
        }
        Some((CancelMarket::DISCRIMINATOR, _)) => CancelMarket::try_from(accounts)?.process(),
        Some((SettleMarket::DISCRIMINATOR, data)) => {
            SettleMarket::try_from((data, accounts))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
