use pinocchio::{
    AccountView, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, rent::Rent},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::CloseAccount;

use crate::utils::constants;

pub struct MintInterface;
impl MintInterface {
    pub fn transfer(
        from: &AccountView,
        to: &AccountView,
        authority: &AccountView,
        amount: u64,
    ) -> ProgramResult {
        pinocchio_token::instructions::Transfer {
            from,
            to,
            authority,
            amount,
        }
        .invoke()
    }

    pub fn transfer_signed(
        from: &AccountView,
        to: &AccountView,
        authority: &AccountView,
        amount: u64,
        seeds: &[Seed],
    ) -> ProgramResult {
        let signer_seeds = [Signer::from(seeds)];

        pinocchio_token::instructions::Transfer {
            from,
            to,
            authority,
            amount,
        }
        .invoke_signed(&signer_seeds)
    }

    pub fn valid_mint_check(mint: &AccountView) -> ProgramResult {
        if mint.address() != constants::WSOL_ADDRESS && mint.address() != constants::USDC_ADDRESS {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub struct TokenAccountInterface;
impl TokenAccountInterface {
    pub fn init_with_seeds(
        account: &AccountView,
        payer: &AccountView,
        owner: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
        seeds: &[Seed],
    ) -> ProgramResult {
        let lamports =
            Rent::get()?.try_minimum_balance(pinocchio_token::state::TokenAccount::LEN)?;

        let signer_seeds = [Signer::from(seeds)];

        CreateAccount {
            from: payer,
            to: account,
            lamports,
            space: pinocchio_token::state::TokenAccount::LEN as u64,
            owner: token_program.address(),
        }
        .invoke_signed(&signer_seeds)?;

        pinocchio_token::instructions::InitializeAccount3 {
            account,
            mint,
            owner: owner.address(),
        }
        .invoke()
    }

    pub fn close_signed(
        account: &AccountView,
        authority: &AccountView,
        destination: &AccountView,
        seeds: &[Seed],
    ) -> ProgramResult {
        let signer_seeds = [Signer::from(seeds)];

        CloseAccount {
            account,
            authority,
            destination,
        }
        .invoke_signed(&signer_seeds)
    }
}
