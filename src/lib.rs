#![cfg_attr(not(test), no_std)]

#[cfg(test)]
mod test;

mod constants;
mod errors;
mod instructions;
mod states;
mod utils;

use pinocchio::{
    error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView,
    Address, ProgramResult,
};

use crate::instructions::Initialize;

nostd_panic_handler!();
no_allocator!();
program_entrypoint!(process_entrypoint);

pinocchio::address::declare_id!("AxXTVNh3eDefaL8F6RVXMKXss77tgUi8MZHQAGrQX5db");

fn process_entrypoint(
    _program_id: &Address,
    accounts: &[AccountView],
    data: &[u8],
) -> ProgramResult {
    match data.split_first() {
        Some((Initialize::DISCRIMINATOR, data)) => {
            Initialize::try_from((data, accounts))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
