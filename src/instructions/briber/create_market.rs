use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::{Config, Market},
    utils::{Account, MintInterface, TokenAccountInterface, constants, math},
};

pub struct CreateMarket<'a> {
    accounts: CreateMarketAccounts<'a>,
    data: CreateMarketData<'a>,
}

pub struct CreateMarketAccounts<'a> {
    authority: &'a AccountView,
    config: &'a AccountView,
    briber: &'a AccountView,
    market: &'a AccountView,
    incentive_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_incentive_vault: &'a AccountView,
    treasury: &'a AccountView,
    outcome_yes_mint: &'a AccountView,
    outcome_no_mint: &'a AccountView,
    market_yes_vault: &'a AccountView,
    market_no_vault: &'a AccountView,
    token_program: &'a AccountView,
    market_incentive_vault_bump: u8,
    market_yes_vault_bump: u8,
    market_no_vault_bump: u8,
}

pub struct CreateMarketData<'a> {
    amount: u64,
    id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for CreateMarketData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // 8 bytes for amount, rest for ID
        if data.len() < constants::MIN_ID_LENGTH + 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let id = &data[8..];

        Ok(Self { amount, id })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for CreateMarketAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            authority,
            config,
            briber,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            treasury,
            outcome_yes_mint, // on-chain validation
            outcome_no_mint,  // on-chain validation
            market_yes_vault,
            market_no_vault,
            token_program,
            _associated_token_program,
            _system_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(authority)?;
        Account::signer_check(briber)?;

        MintInterface::valid_mint_check(incentive_mint)?;

        let (market_incentive_vault_address, market_incentive_vault_bump) =
            Address::find_program_address(
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

        let (treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, incentive_mint.address().as_ref()],
            &crate::ID,
        );
        if &treasury_address != treasury.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (market_yes_vault_address, market_yes_vault_bump) = Address::find_program_address(
            &[
                constants::MARKET_SEED,
                market.address().as_ref(),
                outcome_yes_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        if &market_yes_vault_address != market_yes_vault.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (market_no_vault_address, market_no_vault_bump) = Address::find_program_address(
            &[
                constants::MARKET_SEED,
                market.address().as_ref(),
                outcome_no_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        if &market_no_vault_address != market_no_vault.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            config,
            briber,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            treasury,
            outcome_yes_mint,
            outcome_no_mint,
            market_yes_vault,
            market_no_vault,
            token_program,
            market_incentive_vault_bump,
            market_yes_vault_bump,
            market_no_vault_bump,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for CreateMarket<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: CreateMarketAccounts::try_from(accounts)?,
            data: CreateMarketData::try_from(data)?,
        })
    }
}

impl<'a> CreateMarket<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&self) -> ProgramResult {
        // create market account and set data
        let (fee_bps, briber_fee_bps) = {
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
            (config.fee_bps(), config.briber_fee_bps())
        };

        let (market_address, market_bump) =
            Address::find_program_address(&[constants::MARKET_SEED, self.data.id], &crate::ID);
        if &market_address != self.accounts.market.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let bump_binding = &[market_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.data.id),
            Seed::from(bump_binding),
        ];
        Account::init_pda::<Market>(self.accounts.market, self.accounts.briber, seeds)?;

        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = Market::load_mut(&mut market_data)?;

        market.set_inner(
            self.accounts.briber.address(),
            self.accounts.incentive_mint.address(),
            self.accounts.outcome_yes_mint.address(),
            self.accounts.outcome_no_mint.address(),
            self.data.amount,
            briber_fee_bps,
            market_bump,
        )?;

        // create atas
        // market_incentive_vault
        let bump_binding = &[self.accounts.market_incentive_vault_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(self.accounts.incentive_mint.address().as_ref()),
            Seed::from(bump_binding),
        ];
        TokenAccountInterface::init_with_seeds(
            self.accounts.market_incentive_vault,
            self.accounts.briber,
            self.accounts.market,
            self.accounts.incentive_mint,
            self.accounts.token_program,
            seeds,
        )?;

        // market_yes_vault
        let bump_binding = &[self.accounts.market_yes_vault_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(self.accounts.outcome_yes_mint.address().as_ref()),
            Seed::from(bump_binding),
        ];
        TokenAccountInterface::init_with_seeds(
            self.accounts.market_yes_vault,
            self.accounts.briber,
            self.accounts.market,
            self.accounts.outcome_yes_mint,
            self.accounts.token_program,
            seeds,
        )?;

        // market_no_vault
        let bump_binding = &[self.accounts.market_no_vault_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(self.accounts.outcome_no_mint.address().as_ref()),
            Seed::from(bump_binding),
        ];
        TokenAccountInterface::init_with_seeds(
            self.accounts.market_no_vault,
            self.accounts.briber,
            self.accounts.market,
            self.accounts.outcome_no_mint,
            self.accounts.token_program,
            seeds,
        )?;

        // tranfer fees to treasury
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.treasury,
            self.accounts.briber,
            math::fee_calculation(self.data.amount, fee_bps)?,
        )?;

        // transfer tokens to market incentive vault
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.market_incentive_vault,
            self.accounts.briber,
            self.data.amount,
        )
    }
}
