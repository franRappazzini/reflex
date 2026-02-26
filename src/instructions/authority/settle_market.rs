use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::{Config, MarketVault, MarketVaultResolution},
    utils::Account,
};

pub struct SettleMarket<'a> {
    accounts: SettleMarketAccounts<'a>,
    data: SettleMarketData,
}

pub struct SettleMarketAccounts<'a> {
    // authority: &'a AccountView,
    // config: &'a AccountView, // read only to check authority
    market: &'a AccountView,
}

pub struct SettleMarketData {
    resolution: MarketVaultResolution, // change by on-chain validation when it will be possible
}

impl<'a> TryFrom<&'a [AccountView]> for SettleMarketAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config, market] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(authority)?;

        let (config_address, _) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);

        require_eq_address!(&config_address, config.address());

        let config_data = config.try_borrow()?;
        let config_acc = Config::load(&config_data)?;

        require_eq_address!(&config_acc.authority, authority.address());

        Ok(Self {
            // authority,
            // config,
            market,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for SettleMarketData {
    type Error = ProgramError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        require_eq_len!(bytes.len(), size_of::<Self>());

        let resolution = match bytes[0] {
            1 => MarketVaultResolution::Yes,
            2 => MarketVaultResolution::No,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Self { resolution })
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for SettleMarket<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = SettleMarketAccounts::try_from(accounts)?;
        let data = SettleMarketData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> SettleMarket<'a> {
    pub const DISCRIMINATOR: &'a u8 = &6;

    pub fn process(&mut self) -> ProgramResult {
        let mut market_data = self.accounts.market.try_borrow_mut()?;

        let market = MarketVault::load_mut(&mut market_data)?;

        if market.is_settled() {
            return Err(ReflexError::MarketWasSetted.into());
        }

        market.set_as_settled();
        market.set_resolution(self.data.resolution);

        Ok(())
    }
}
