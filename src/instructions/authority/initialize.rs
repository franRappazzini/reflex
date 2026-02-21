use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::Config,
    utils::{Account, MintInterface, TokenAcocuntInterface},
};

pub struct Initialize<'a> {
    accounts: InitializeAccounts<'a>,
    data: InitializeData,
}

pub struct InitializeAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
    wsol_mint: &'a AccountView,
    usdc_mint: &'a AccountView,
    wsol_treasury: &'a AccountView,
    usdc_treasury: &'a AccountView,
    bump: u8,
    bump_wsol_treasury: u8,
    bump_usdc_treasury: u8,
    token_program: &'a AccountView,
}

pub struct InitializeData {
    fee_bps: u16,
    briber_fee_bps: u16,
}

impl<'a> TryFrom<&'a [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [authority, config, wsol_mint, usdc_mint, wsol_treasury, usdc_treasury, token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(authority)?;
        Account::not_initialized_check(config)?;
        MintInterface::check(wsol_mint)?; // TODO: devnet/mainnet change to harcoded addresses
        MintInterface::check(usdc_mint)?; // TODO: devnet/mainnet change to harcoded addresses
        Account::not_initialized_check(wsol_treasury)?;
        Account::not_initialized_check(usdc_treasury)?;

        let (config_address, bump) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);
        require_eq_address!(&config_address, config.address());

        let (wsol_treasury_address, bump_wsol_treasury) = Address::find_program_address(
            &[constants::TREASURY_SEED, wsol_mint.address().as_ref()],
            &crate::ID,
        );
        require_eq_address!(&wsol_treasury_address, wsol_treasury.address());

        let (usdc_treasury_address, bump_usdc_treasury) = Address::find_program_address(
            &[constants::TREASURY_SEED, usdc_mint.address().as_ref()],
            &crate::ID,
        );
        require_eq_address!(&usdc_treasury_address, usdc_treasury.address());

        Ok(Self {
            authority,
            config,
            wsol_mint,
            usdc_mint,
            wsol_treasury,
            usdc_treasury,
            bump,
            bump_wsol_treasury,
            bump_usdc_treasury,
            token_program,
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
        // create and set config account
        let bump_binding = [self.accounts.bump];
        let seeds = [
            Seed::from(constants::CONFIG_SEED),
            Seed::from(&bump_binding),
        ];

        Account::init_pda::<Config>(self.accounts.authority, self.accounts.config, &seeds)?;

        {
            let mut config_bytes = self.accounts.config.try_borrow_mut()?;
            let config = Config::load_mut(&mut config_bytes)?;

            config.set_inner(
                self.accounts.authority.address().clone(),
                self.data.fee_bps,
                self.data.briber_fee_bps,
                self.accounts.bump,
            );
        }

        // create wsol treasury
        let bump_binding = [self.accounts.bump_wsol_treasury];
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.wsol_mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        TokenAcocuntInterface::init_with_seeds(
            self.accounts.wsol_treasury,
            self.accounts.authority,
            self.accounts.config,
            self.accounts.wsol_mint,
            self.accounts.token_program,
            &seeds,
        )?;

        // create usdc treasury
        let bump_binding = [self.accounts.bump_usdc_treasury];
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.usdc_mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        TokenAcocuntInterface::init_with_seeds(
            self.accounts.usdc_treasury,
            self.accounts.authority,
            self.accounts.config,
            self.accounts.usdc_mint,
            self.accounts.token_program,
            &seeds,
        )
    }
}
