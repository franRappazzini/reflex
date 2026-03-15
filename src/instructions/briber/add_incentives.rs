use pinocchio::{AccountView, Address, ProgramResult, error::ProgramError};

use crate::{
    states::{Config, Market},
    utils::{Account, MintInterface, constants, math},
};

pub struct AddIncentives<'a> {
    accounts: AddIncentivesAccounts<'a>,
    data: AddIncentivesData<'a>,
}

pub struct AddIncentivesAccounts<'a> {
    briber: &'a AccountView,
    config: &'a AccountView,
    treasury: &'a AccountView,
    market: &'a AccountView,
    incentive_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_incentive_vault: &'a AccountView,
}

pub struct AddIncentivesData<'a> {
    amount: u64,
    id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for AddIncentivesData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        let amount = u64::from_le_bytes(data[..8].try_into().unwrap());
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            amount,
            id: &data[8..],
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for AddIncentivesAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            briber,
            config,
            treasury,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            _token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(briber)?;

        let (config_address, _) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);
        if &config_address != config.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, incentive_mint.address().as_ref()],
            &crate::ID,
        );
        if &treasury_address != treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (market_incentive_vault_address, _) = Address::find_program_address(
            &[
                constants::MARKET_SEED,
                market.address().as_ref(),
                incentive_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        if &market_incentive_vault_address != market_incentive_vault.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            briber,
            config,
            treasury,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for AddIncentives<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: accounts.try_into()?,
            data: data.try_into()?,
        })
    }
}

impl<'a> AddIncentives<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&self) -> ProgramResult {
        // check market, briber and incentive mint are valid
        let (market_address, _) =
            Address::find_program_address(&[constants::MARKET_SEED, self.data.id], &crate::ID);
        if &market_address != self.accounts.market.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = Market::load_mut(&mut market_data)?;

        if !market.is_open() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &market.briber() != self.accounts.briber.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &market.incentive_mint() != self.accounts.incentive_mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        // update market data
        market.add_incentives(self.data.amount)?;

        let config_data = self.accounts.config.try_borrow()?;
        let config = Config::load(&config_data)?;

        // transfer fees to treasury
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.treasury,
            self.accounts.briber,
            math::fee_calculation(self.data.amount, config.fee_bps())?,
        )?;

        // transfer incentives to market
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.market_incentive_vault,
            self.accounts.briber,
            self.data.amount,
        )
    }
}
