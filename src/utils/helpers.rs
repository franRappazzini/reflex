use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub struct Account;
impl Account {
    pub fn _program_account_check(account: &AccountView) -> ProgramResult {
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(())
    }

    pub fn system_program_check(account: &AccountView) -> ProgramResult {
        if !account.owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(())
    }

    pub fn signer_check(account: &AccountView) -> ProgramResult {
        if !account.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }

    pub fn not_initialized_check(account: &AccountView) -> ProgramResult {
        Self::system_program_check(account)?;

        if account.lamports() > 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }

    pub fn init_pda<T>(from: &AccountView, to: &AccountView, seeds: &[Seed]) -> ProgramResult {
        let space = size_of::<T>();
        let lamports = Rent::get()?.try_minimum_balance(space)?;

        let signer_seeds = [Signer::from(seeds)];

        CreateAccount {
            from,
            to,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(&signer_seeds)
    }
}
