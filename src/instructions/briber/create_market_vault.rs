use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::{Config, MarketVault},
    utils::{fee_calculation, Account, MintInterface, TokenAcocuntInterface},
};

pub struct CreateMarketVault<'a> {
    accounts: CreateMarketVaultAccounts<'a>,
    data: CreateMarketVaultData,
}

pub struct CreateMarketVaultAccounts<'a> {
    briber: &'a AccountView,
    config: &'a AccountView,
    incentive_mint: &'a AccountView,
    treasury: &'a AccountView,
    market_vault: &'a AccountView,
    outcome_yes_mint: &'a AccountView,
    outcome_no_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_vault_treasury: &'a AccountView,
    outcome_yes_vault: &'a AccountView,
    outcome_no_vault: &'a AccountView,
    token_program: &'a AccountView,
    market_vault_treasury_bump: u8,
    bump_outcome_yes_vault: u8,
    bump_outcome_no_vault: u8,
}

pub struct CreateMarketVaultData {
    amount: u64,
}

impl<'a> TryFrom<&'a [AccountView]> for CreateMarketVaultAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [briber, config, incentive_mint, treasury, market_vault, outcome_yes_mint, outcome_no_mint, briber_ata, market_vault_treasury, outcome_yes_vault, outcome_no_vault, token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(briber)?;
        Account::program_account_check(config)?;
        MintInterface::check(incentive_mint)?; // TODO: check usdc/wsol
        TokenAcocuntInterface::token_account_check(treasury, config, incentive_mint)?;
        Account::not_initialized_check(market_vault)?;
        MintInterface::check(outcome_yes_mint)?;
        MintInterface::check(outcome_no_mint)?;
        TokenAcocuntInterface::check(briber_ata)?;
        Account::not_initialized_check(market_vault_treasury)?;
        Account::not_initialized_check(outcome_yes_vault)?;
        Account::not_initialized_check(outcome_no_vault)?;

        let (config_address, _) =
            Address::find_program_address(&[constants::CONFIG_SEED], &crate::ID);

        require_eq_address!(&config_address, config.address());

        let (outcome_yes_vault_address, bump_outcome_yes_vault) = Address::find_program_address(
            &[
                constants::TREASURY_SEED,
                market_vault.address().as_ref(),
                outcome_yes_mint.address().as_ref(),
            ],
            &crate::ID,
        );

        require_eq_address!(&outcome_yes_vault_address, outcome_yes_vault.address());

        let (outcome_no_vault_address, bump_outcome_no_vault) = Address::find_program_address(
            &[
                constants::TREASURY_SEED,
                market_vault.address().as_ref(),
                outcome_no_mint.address().as_ref(),
            ],
            &crate::ID,
        );

        require_eq_address!(&outcome_no_vault_address, outcome_no_vault.address());

        let (market_vault_treasury_address, market_vault_treasury_bump) =
            Address::find_program_address(
                &[constants::TREASURY_SEED, &market_vault.address().as_ref()],
                &crate::ID,
            );

        require_eq_address!(
            &market_vault_treasury_address,
            market_vault_treasury.address()
        );

        Ok(Self {
            briber,
            config,
            incentive_mint,
            treasury,
            market_vault,
            outcome_yes_mint,
            outcome_no_mint,
            briber_ata,
            market_vault_treasury,
            outcome_yes_vault,
            outcome_no_vault,
            token_program,
            market_vault_treasury_bump,
            bump_outcome_yes_vault,
            bump_outcome_no_vault,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for CreateMarketVaultData {
    type Error = ProgramError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        require_eq_len!(bytes.len(), size_of::<Self>());

        let amount = u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );

        Ok(Self { amount })
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for CreateMarketVault<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = CreateMarketVaultAccounts::try_from(accounts)?;
        let data = CreateMarketVaultData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> CreateMarketVault<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        // create market vault account
        let mut config_data = self.accounts.config.try_borrow_mut()?;
        let config: &mut Config = Config::load_mut(&mut config_data)?;
        config.add_market_counter()?;

        let market_counter_binding = config.market_counter().to_le_bytes();

        let (market_vault_address, market_vault_bump) = Address::find_program_address(
            &[constants::MARKET_VAULT_SEED, &market_counter_binding],
            &crate::ID,
        );

        require_eq_address!(&market_vault_address, self.accounts.market_vault.address());

        let bump_binding = market_vault_bump.to_le_bytes();
        let seeds = [
            Seed::from(constants::MARKET_VAULT_SEED),
            Seed::from(&market_counter_binding),
            Seed::from(&bump_binding),
        ];

        Account::init_pda::<MarketVault>(self.accounts.briber, self.accounts.market_vault, &seeds)?;

        // create market vault treasury ata
        let bump_binding = self.accounts.market_vault_treasury_bump.to_le_bytes();
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(market_vault_address.as_ref()),
            Seed::from(&bump_binding),
        ];

        TokenAcocuntInterface::init_with_seeds(
            self.accounts.market_vault_treasury,
            self.accounts.briber,
            self.accounts.market_vault,
            self.accounts.incentive_mint,
            self.accounts.token_program,
            &seeds,
        )?;

        // create outcome yes vault ata
        let bump_binding = self.accounts.bump_outcome_yes_vault.to_le_bytes();
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(market_vault_address.as_ref()),
            Seed::from(self.accounts.outcome_yes_mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        TokenAcocuntInterface::init_with_seeds(
            self.accounts.outcome_yes_vault,
            self.accounts.briber,
            self.accounts.market_vault,
            self.accounts.outcome_yes_mint,
            self.accounts.token_program,
            &seeds,
        )?;

        // create outcome no vault ata
        let bump_binding = self.accounts.bump_outcome_no_vault.to_le_bytes();
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(market_vault_address.as_ref()),
            Seed::from(self.accounts.outcome_no_mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        TokenAcocuntInterface::init_with_seeds(
            self.accounts.outcome_no_vault,
            self.accounts.briber,
            self.accounts.market_vault,
            self.accounts.outcome_no_mint,
            self.accounts.token_program,
            &seeds,
        )?;

        // transfer incentive fee to program treasury
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.treasury,
            self.accounts.briber,
            fee_calculation(self.data.amount, config.fee_bps())?, // fee calculation
        )?;

        // transfer incentive rewards to market treasury
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.market_vault_treasury,
            self.accounts.briber,
            self.data.amount,
        )?;

        // set market vault
        let mut market_vault_data = self.accounts.market_vault.try_borrow_mut()?;
        let market_vault = MarketVault::load_mut(&mut market_vault_data)?;

        market_vault.set_inner(
            config.market_counter(),
            self.accounts.briber.address(),
            self.accounts.outcome_yes_mint.address(),
            self.accounts.outcome_no_mint.address(),
            self.accounts.incentive_mint.address(),
            config.briber_fee_bps(),
            market_vault_bump,
            self.data.amount,
        );

        Ok(())
    }
}
