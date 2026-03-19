use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};
use pinocchio_token::state::TokenAccount;

use crate::{
    states::Config,
    utils::{Account, MintInterface, constants},
};

pub struct WithdrawTreasury<'a> {
    accounts: WithdrawTreasuryAccounts<'a>,
}

struct WithdrawTreasuryAccounts<'a> {
    config: &'a AccountView,
    wsol_treasury: &'a AccountView,
    usdc_treasury: &'a AccountView,
    wsol_destination: &'a AccountView,
    usdc_destination: &'a AccountView,
    config_bump: u8,
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawTreasuryAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            authority,
            config,
            wsol_mint,
            usdc_mint,
            wsol_treasury,
            usdc_treasury,
            wsol_destination,
            usdc_destination,
            _token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(authority)?;

        let config_data = config.try_borrow()?;
        let config_account = Config::load(&config_data)?;

        let config_address = Address::derive_address(
            &[constants::CONFIG_SEED],
            Some(config_account.bump),
            &crate::ID,
        );

        if &config_address != config.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &config_account.authority() != authority.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (wsol_treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, wsol_mint.address().as_ref()],
            &crate::ID,
        );
        if &wsol_treasury_address != wsol_treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (usdc_treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, usdc_mint.address().as_ref()],
            &crate::ID,
        );
        if &usdc_treasury_address != usdc_treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            config,
            wsol_treasury,
            usdc_treasury,
            wsol_destination,
            usdc_destination,
            config_bump: config_account.bump,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for WithdrawTreasury<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: WithdrawTreasuryAccounts::try_from(accounts)?,
        })
    }
}

impl<'a> WithdrawTreasury<'a> {
    pub const DISCRIMINATOR: &'a u8 = &6;

    pub fn process(&self) -> ProgramResult {
        let bump_binding = &[self.accounts.config_bump];
        let seeds = &[Seed::from(constants::CONFIG_SEED), Seed::from(bump_binding)];

        // transfer wsol
        let amount = { TokenAccount::from_account_view(self.accounts.wsol_treasury)?.amount() };

        MintInterface::transfer_signed(
            self.accounts.wsol_treasury,
            self.accounts.wsol_destination,
            self.accounts.config,
            amount,
            seeds,
        )?;

        // transfer usdc
        let amount = { TokenAccount::from_account_view(self.accounts.usdc_treasury)?.amount() };

        MintInterface::transfer_signed(
            self.accounts.usdc_treasury,
            self.accounts.usdc_destination,
            self.accounts.config,
            amount,
            seeds,
        )
    }
}
