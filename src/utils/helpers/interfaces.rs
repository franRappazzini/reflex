use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

use crate::{errors::ReflexError, require_eq_address, require_eq_len};

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb
pub const TOKEN_2022_PROGRAM_ID: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];
pub const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;

pub struct MintInterface;
impl MintInterface {
    pub fn check(account: &AccountView) -> ProgramResult {
        if account.owned_by(&pinocchio_token::ID) {
            // legacy spl
            require_eq_len!(account.data_len(), pinocchio_token::state::Mint::LEN);
        } else if account.owned_by(&Address::new_from_array(TOKEN_2022_PROGRAM_ID)) {
            // token2022
            let data = account.try_borrow()?;
            if data.len() <= TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET] != TOKEN_2022_MINT_DISCRIMINATOR {
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }

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
}

pub struct TokenAcocuntInterface;
impl TokenAcocuntInterface {
    pub fn check(account: &AccountView) -> ProgramResult {
        if account.owned_by(&pinocchio_token::ID) {
            // legacy spl
            require_eq_len!(
                account.data_len(),
                pinocchio_token::state::TokenAccount::LEN
            );
        } else if account.owned_by(&Address::new_from_array(TOKEN_2022_PROGRAM_ID)) {
            // token2022
            let data = account.try_borrow()?;
            if data.len() <= TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET {
                return Err(ProgramError::InvalidAccountData);
            }
            if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET]
                != TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR
            {
                return Err(ProgramError::InvalidAccountData);
            }
        } else {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(())
    }

    pub fn token_account_check(
        account: &AccountView,   // treasury
        authority: &AccountView, // config
        mint: &AccountView,
    ) -> ProgramResult {
        let ata = pinocchio_token::state::TokenAccount::from_account_view(account)?;
        require_eq_address!(ata.owner(), authority.address());
        require_eq_address!(ata.mint(), mint.address());

        Ok(())
    }

    pub fn ata_check(
        account: &AccountView,
        owner: &AccountView,
        mint: &AccountView,
        token_program: &AccountView,
    ) -> ProgramResult {
        require_eq_address!(
            &Address::find_program_address(
                &[
                    owner.address().as_ref(),
                    token_program.address().as_ref(),
                    mint.address().as_ref(),
                ],
                &pinocchio_associated_token_account::ID
            )
            .0,
            account.address()
        );

        Ok(())
    }

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
}
