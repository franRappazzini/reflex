use pinocchio::{AccountView, Address, ProgramResult, error::ProgramError};

use crate::{
    states::Config,
    utils::{Account, constants},
};

pub struct UpdateConfig<'a> {
    accounts: UpdateConfigAccounts<'a>,
    data: UpdateConfigData,
}

struct UpdateConfigAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
}

struct UpdateConfigData {
    new_authority: Address,
    new_fee_bps: u16,
    new_briber_fee_bps: u16,
}

impl<'a> TryFrom<&'a [u8]> for UpdateConfigData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        let new_authority = Address::new_from_array(data[0..32].try_into().unwrap());
        let new_fee_bps = u16::from_le_bytes(data[32..34].try_into().unwrap());
        let new_briber_fee_bps = u16::from_le_bytes(data[34..36].try_into().unwrap());

        Ok(Self {
            new_authority,
            new_fee_bps,
            new_briber_fee_bps,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for UpdateConfigAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(authority)?;

        Ok(Self { authority, config })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for UpdateConfig<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: UpdateConfigAccounts::try_from(accounts)?,
            data: UpdateConfigData::try_from(data)?,
        })
    }
}

impl<'a> UpdateConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &10;

    pub fn process(&self) -> ProgramResult {
        // check config and authority
        let mut config_data = self.accounts.config.try_borrow_mut()?;
        let config = Config::load_mut(&mut config_data)?;

        let config_address =
            Address::derive_address(&[constants::CONFIG_SEED], Some(config.bump), &crate::ID);
        if &config_address != self.accounts.config.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if &config.authority() != self.accounts.authority.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        // update config
        config.update(
            &self.data.new_authority,
            self.data.new_fee_bps,
            self.data.new_briber_fee_bps,
        );

        Ok(())
    }
}
