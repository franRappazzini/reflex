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

    pub fn program_account_check(account: &AccountView) -> ProgramResult {
        if !account.owned_by(&crate::ID) {
            return Err(ProgramError::InvalidAccountData);
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

    pub fn init_if_needed<T>(
        account: &AccountView,
        payer: &AccountView,
        seeds: &[Seed],
    ) -> ProgramResult {
        match Self::program_account_check(account) {
            Ok(_) => Ok(()),
            Err(_) => Self::init_pda::<T>(account, payer, seeds),
        }
    }

    pub fn close(account: &AccountView, destination: &AccountView) -> ProgramResult {
        {
            let mut data = account.try_borrow_mut()?;
            data[0] = 0xff;
        }

        destination.set_lamports(destination.lamports() + account.lamports());
        account.set_lamports(0);

        account.resize(1)?;
        account.close()
    }
}
