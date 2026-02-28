use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants, errors::ReflexError, require_eq_address, require_eq_len, states::Config,
    utils::Account,
};

pub struct UpdateConfig<'a> {
    accounts: UpdateConfigAccounts<'a>,
    data: UpdateConfigData,
}

pub struct UpdateConfigAccounts<'a> {
    // authority: &'a AccountView,
    config: &'a AccountView,
}

pub struct UpdateConfigData {
    new_fee_bps: u16,
    new_briber_fee_bps: u16,
}

impl<'a> TryFrom<&'a [AccountView]> for UpdateConfigAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(authority)?;

        let (config_address, _) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);
        require_eq_address!(&config_address, config.address());

        Ok(Self { config })
    }
}

impl<'a> TryFrom<&'a [u8]> for UpdateConfigData {
    type Error = ProgramError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        require_eq_len!(bytes.len(), size_of::<Self>());

        let new_fee_bps = u16::from_le_bytes(
            bytes[..2]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        let new_briber_fee_bps = u16::from_le_bytes(
            bytes[2..4]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Self {
            new_fee_bps,
            new_briber_fee_bps,
        })
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for UpdateConfig<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = UpdateConfigAccounts::try_from(accounts)?;
        let data = UpdateConfigData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> UpdateConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &7;

    pub fn process(&mut self) -> ProgramResult {
        let mut config_data = self.accounts.config.try_borrow_mut()?;
        let config = Config::load_mut(&mut config_data)?;

        config.set_fee_bps(self.data.new_fee_bps);
        config.set_briber_fee_bps(self.data.new_briber_fee_bps);

        Ok(())
    }
}
