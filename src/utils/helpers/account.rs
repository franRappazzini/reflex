use pinocchio::cpi::{Seed, Signer};
use pinocchio::sysvars::Sysvar;
use pinocchio::{AccountView, ProgramResult, error::ProgramError, sysvars::rent::Rent};
use pinocchio_system::instructions::CreateAccount;

pub struct Account;
impl Account {
    pub fn signer_check(account: &AccountView) -> ProgramResult {
        if !account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }

    pub fn not_initialized_check(account: &AccountView) -> ProgramResult {
        if !account.is_data_empty()
            || account.lamports() > 0
            || !account.owned_by(&pinocchio_system::ID)
        {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        Ok(())
    }

    pub fn init_pda<T>(
        account: &AccountView,
        payer: &AccountView,
        seeds: &[Seed],
    ) -> ProgramResult {
        let space = size_of::<T>();
        let lamports = Rent::get()?.try_minimum_balance(space)?;

        let signer_seeds = &[Signer::from(seeds)];

        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: space as u64,
            owner: &crate::ID,
        }
        .invoke_signed(signer_seeds)
    }
}
