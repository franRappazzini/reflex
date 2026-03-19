use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};
use pinocchio_token::state::TokenAccount;

use crate::{
    states::Market,
    utils::{Account, MintInterface, TokenAccountInterface, constants},
};

pub struct CancelMarket<'a> {
    accounts: CancelMarketAccounts<'a>,
    data: CancelMarketData<'a>,
}

struct CancelMarketAccounts<'a> {
    briber: &'a AccountView,
    market: &'a AccountView,
    incentive_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_incentive_vault: &'a AccountView,
    market_yes_vault: &'a AccountView,
    market_no_vault: &'a AccountView,
}

struct CancelMarketData<'a> {
    id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for CancelMarketData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { id: data })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for CancelMarketAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            briber,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            market_yes_vault,
            market_no_vault,
            _token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(briber)?;

        Ok(Self {
            briber,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            market_yes_vault,
            market_no_vault,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for CancelMarket<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: CancelMarketAccounts::try_from(accounts)?,
            data: CancelMarketData::try_from(data)?,
        })
    }
}

impl<'a> CancelMarket<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(&self) -> ProgramResult {
        // check market and data, briber, incentive mint
        let (total_incentive_amount, market_bump) = {
            let market_data = self.accounts.market.try_borrow()?;
            let market = Market::load(&market_data)?;

            let market_address = Address::derive_address(
                &[constants::MARKET_SEED, self.data.id],
                Some(market.bump),
                &crate::ID,
            );
            if &market_address != self.accounts.market.address() {
                return Err(ProgramError::InvalidAccountData);
            }
            // just check available fees because staked tokens can be withdrawn by users but fees can only when market is settled
            if market.available_yes_fees() > 0 || market.available_no_fees() > 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            if &market.briber() != self.accounts.briber.address() {
                return Err(ProgramError::InvalidAccountData);
            }
            if &market.incentive_mint() != self.accounts.incentive_mint.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            (market.total_incentive_amount(), market.bump)
        };

        // transfer back to briber ata
        let bump_binding = &[market_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.data.id),
            Seed::from(bump_binding),
        ];

        MintInterface::transfer_signed(
            self.accounts.market_incentive_vault,
            self.accounts.briber_ata,
            self.accounts.market,
            total_incentive_amount,
            seeds,
        )?;

        // close accounts
        let (market_yes_vault_amount, market_no_vault_amount) = {
            (
                TokenAccount::from_account_view(self.accounts.market_yes_vault)?.amount(),
                TokenAccount::from_account_view(self.accounts.market_no_vault)?.amount(),
            )
        };

        if market_yes_vault_amount == 0 {
            TokenAccountInterface::close_signed(
                self.accounts.market_yes_vault,
                self.accounts.market,
                self.accounts.briber,
                seeds,
            )?;
        }
        if market_no_vault_amount == 0 {
            TokenAccountInterface::close_signed(
                self.accounts.market_no_vault,
                self.accounts.market,
                self.accounts.briber,
                seeds,
            )?;
        }
        TokenAccountInterface::close_signed(
            self.accounts.market_incentive_vault,
            self.accounts.market,
            self.accounts.briber,
            seeds,
        )?;
        Account::close(self.accounts.market, self.accounts.briber)
    }
}
