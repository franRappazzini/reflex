use pinocchio::{AccountView, Address, ProgramResult, error::ProgramError};

use crate::{
    states::{Config, Market},
    utils::{Account, constants},
};

pub struct SettleMarket<'a> {
    accounts: SettleMarketAccounts<'a>,
    data: SettleMarketData<'a>,
}

struct SettleMarketAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
    market: &'a AccountView,
}

#[repr(C, packed)]
struct SettleMarketData<'a> {
    resolution: u8, // 1 = Yes, 2 = No
    id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for SettleMarketData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        let resolution = data[0];
        if resolution != 1 && resolution != 2 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let id = &data[1..];

        Ok(Self { id, resolution })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for SettleMarketAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config, market] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(authority)?;

        Ok(Self {
            authority,
            config,
            market,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for SettleMarket<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: accounts.try_into()?,
            data: data.try_into()?,
        })
    }
}

impl<'a> SettleMarket<'a> {
    pub const DISCRIMINATOR: &'a u8 = &5;

    pub fn process(&self) -> ProgramResult {
        // check market and authority
        let config_data = self.accounts.config.try_borrow()?;
        let config = Config::load(&config_data)?;

        let config_address =
            Address::derive_address(&[constants::CONFIG_SEED], Some(config.bump), &crate::ID);
        if &config_address != self.accounts.config.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &config.authority() != self.accounts.authority.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = Market::load_mut(&mut market_data)?;

        let market_address = Address::derive_address(
            &[constants::MARKET_SEED, self.data.id],
            Some(market.bump),
            &crate::ID,
        );
        if &market_address != self.accounts.market.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        market.set_resolution(self.data.resolution);

        Ok(())
    }
}
