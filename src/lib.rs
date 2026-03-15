#![no_std]

mod instructions;
mod states;
mod utils;

use pinocchio::{
    AccountView, Address, ProgramResult, address::declare_id, error::ProgramError, no_allocator,
    nostd_panic_handler, program_entrypoint,
};

use crate::instructions::{AddIncentives, CreateMarket, Initialize};

no_allocator!();
nostd_panic_handler!();
program_entrypoint!(process_instruction);

declare_id!("4ZegtDo8WG6e2PAswLhnGXYDS5TGkniVCKXDrDX12KYX");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    ix_data: &[u8],
) -> ProgramResult {
    match ix_data.split_first() {
        Some((Initialize::DISCRIMINATOR, data)) => {
            Initialize::try_from((accounts, data))?.process()
        }
        Some((CreateMarket::DISCRIMINATOR, data)) => {
            CreateMarket::try_from((accounts, data))?.process()
        }
        Some((AddIncentives::DISCRIMINATOR, data)) => {
            AddIncentives::try_from((accounts, data))?.process()
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
