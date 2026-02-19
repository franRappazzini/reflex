use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants, errors::ReflexError, require_eq_address, require_eq_len, states::Config,
    utils::Account,
};

pub struct Initialize<'a> {
    accounts: InitializeAccounts<'a>,
    data: InitializeData,
}

pub struct InitializeAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
    bump: u8,
}

pub struct InitializeData {
    fee_bps: u16,
    briber_fee_bps: u16,
}

impl<'a> TryFrom<&'a [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config, _system_program] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(authority)?;
        Account::not_initialized_check(config)?;

        let (config_address, bump) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);

        require_eq_address!(&config_address, config.address());

        Ok(Self {
            authority,
            config,
            bump,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for InitializeData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        require_eq_len!(data.len(), size_of::<InitializeData>());

        let fee_bps = u16::from_le_bytes(
            data[0..2]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        let briber_fee_bps = u16::from_le_bytes(
            data[2..4]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Self {
            fee_bps,
            briber_fee_bps,
        })
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for Initialize<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = InitializeAccounts::try_from(accounts)?;
        let data = InitializeData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        let bump_binding = &self.accounts.bump.to_le_bytes();
        let seeds = [Seed::from(constants::CONFIG_SEED), Seed::from(bump_binding)];

        Account::init_pda::<Config>(self.accounts.authority, self.accounts.config, &seeds)?;

        let mut config_bytes = self.accounts.config.try_borrow_mut()?;
        let config = Config::load_mut(&mut config_bytes)?;

        config.set_inner(
            self.accounts.authority.address().clone(),
            self.data.fee_bps,
            self.data.briber_fee_bps,
            self.accounts.bump,
        );

        Ok(())
    }
}
