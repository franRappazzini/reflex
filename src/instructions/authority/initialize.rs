use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::Config,
    utils::{Account, MintInterface, TokenAccountInterface, constants},
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
    token_program: &'a AccountView,
    config_bump: u8,
    wsol_treasury_bump: u8,
    usdc_treasury_bump: u8,
}

pub struct InitializeData {
    fee_bps: u16,
    briber_fee_bps: u16,
}

// --- IMPLEMENTATIONS ---

impl<'a> TryFrom<&'a [u8]> for InitializeData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        let fee_bps = u16::from_le_bytes(data[..2].try_into().unwrap());
        let briber_fee_bps = u16::from_le_bytes(data[2..4].try_into().unwrap());

        if fee_bps > 10_000 || briber_fee_bps > 10_000 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            fee_bps,
            briber_fee_bps,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            authority,
            config,
            wsol_mint,
            usdc_mint,
            wsol_treasury,
            usdc_treasury,
            token_program,
            _system_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(authority)?;
        MintInterface::valid_mint_check(wsol_mint)?;
        MintInterface::valid_mint_check(usdc_mint)?;

        let (config_address, config_bump) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);
        if &config_address != config.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (wsol_treasury_address, wsol_treasury_bump) = Address::find_program_address(
            &[constants::TREASURY_SEED, wsol_mint.address().as_ref()],
            &crate::ID,
        );
        if &wsol_treasury_address != wsol_treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (usdc_treasury_address, usdc_treasury_bump) = Address::find_program_address(
            &[constants::TREASURY_SEED, usdc_mint.address().as_ref()],
            &crate::ID,
        );
        if &usdc_treasury_address != usdc_treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            config,
            wsol_mint,
            usdc_mint,
            wsol_treasury,
            usdc_treasury,
            token_program,
            config_bump,
            wsol_treasury_bump,
            usdc_treasury_bump,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for Initialize<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: accounts.try_into()?,
            data: data.try_into()?,
        })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        // create config account and set data
        let bump_binding = &[self.accounts.config_bump];
        let seeds = &[Seed::from(constants::CONFIG_SEED), Seed::from(bump_binding)];
        Account::init_pda::<Config>(self.accounts.config, self.accounts.authority, seeds)?;

        let mut config_data = self.accounts.config.try_borrow_mut()?;
        let config = Config::load_mut(&mut config_data)?;

        config.set_inner(
            self.accounts.authority.address(),
            self.data.fee_bps,
            self.data.briber_fee_bps,
            self.accounts.config_bump,
        );

        // initialize wsol treasury
        let bump_binding = &[self.accounts.wsol_treasury_bump];
        let seeds = &[
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.wsol_mint.address().as_ref()),
            Seed::from(bump_binding),
        ];

        TokenAccountInterface::init_with_seeds(
            self.accounts.wsol_treasury,
            self.accounts.authority,
            self.accounts.config,
            self.accounts.wsol_mint,
            self.accounts.token_program,
            seeds,
        )?;

        // initialize usdc treasury
        let bump_binding = &[self.accounts.usdc_treasury_bump];
        let seeds = &[
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.usdc_mint.address().as_ref()),
            Seed::from(bump_binding),
        ];

        TokenAccountInterface::init_with_seeds(
            self.accounts.usdc_treasury,
            self.accounts.authority,
            self.accounts.config,
            self.accounts.usdc_mint,
            self.accounts.token_program,
            seeds,
        )?;

        Ok(())
    }
}
