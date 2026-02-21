use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::{FarmerPosition, MarketVault},
    utils::{fee_calculation, Account, MintInterface, TokenAcocuntInterface},
};

pub struct StakeOutcomeToken<'a> {
    accounts: StakeOutcomeTokenAccounts<'a>,
    data: StakeOutcomeTokenData,
}

pub struct StakeOutcomeTokenAccounts<'a> {
    farmer: &'a AccountView,
    market_vault: &'a AccountView,
    farmer_position: &'a AccountView,
    outcome_mint: &'a AccountView,
    market_outcome_vault: &'a AccountView,
    farmer_outcome_ata: &'a AccountView,
    bump_farmer_position: u8,
}

pub struct StakeOutcomeTokenData {
    amount: u64,
}

impl<'a> TryFrom<&'a [AccountView]> for StakeOutcomeTokenAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [farmer, market_vault, farmer_position, outcome_mint, market_outcome_vault, farmer_outcome_ata, token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(farmer)?;
        Account::program_account_check(market_vault)?;
        MintInterface::check(outcome_mint)?;
        TokenAcocuntInterface::token_account_check(
            market_outcome_vault,
            market_vault,
            outcome_mint,
        )?;
        TokenAcocuntInterface::ata_check(farmer_outcome_ata, farmer, outcome_mint, token_program)?;

        let (farmer_position_address, farmer_position_bump) = Address::find_program_address(
            &[
                constants::FARMER_POSITION_SEED,
                market_vault.address().as_ref(),
                farmer.address().as_ref(),
            ],
            &crate::ID,
        );

        require_eq_address!(&farmer_position_address, farmer_position.address());

        Ok(Self {
            farmer,
            market_vault,
            farmer_position,
            outcome_mint,
            market_outcome_vault,
            farmer_outcome_ata,
            bump_farmer_position: farmer_position_bump,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for StakeOutcomeTokenData {
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

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for StakeOutcomeToken<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = StakeOutcomeTokenAccounts::try_from(accounts)?;
        let data = StakeOutcomeTokenData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> StakeOutcomeToken<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        // check valid outcome_mint in market_vault
        let mut market_vault_data = self.accounts.market_vault.try_borrow_mut()?;
        let market_vault = MarketVault::load_mut(&mut market_vault_data)?;

        if &market_vault.outcome_yes_mint != self.accounts.outcome_mint.address()
            && &market_vault.outcome_no_mint != self.accounts.outcome_mint.address()
        {
            return Err(ReflexError::InvalidOutcomeMint.into());
        }

        // init if needed farmer position
        let bump_binding = self.accounts.bump_farmer_position.to_le_bytes();
        let seeds = [
            Seed::from(constants::FARMER_POSITION_SEED),
            Seed::from(self.accounts.market_vault.address().as_ref()),
            Seed::from(self.accounts.farmer.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        Account::init_if_needed::<FarmerPosition>(
            self.accounts.farmer_position,
            self.accounts.farmer,
            &seeds,
        )?;

        // transfer
        MintInterface::transfer(
            self.accounts.farmer_outcome_ata,
            self.accounts.market_outcome_vault,
            self.accounts.farmer,
            self.data.amount,
        )?;

        // update farmer_position and market_vault
        let mut farmer_position_data = self.accounts.farmer_position.try_borrow_mut()?;
        let farmer_position = FarmerPosition::load_mut(&mut farmer_position_data)?;

        if !farmer_position.is_initialized {
            farmer_position.set_inner(self.accounts.farmer, self.accounts.bump_farmer_position);
        }

        let fees = fee_calculation(self.data.amount, market_vault.fee_bps())?;
        let staked_amount = self
            .data
            .amount
            .checked_sub(fees)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        if self.accounts.outcome_mint.address() == &market_vault.outcome_yes_mint {
            // yes mint
            market_vault.add_total_yes_staked(staked_amount)?;
            market_vault.add_total_yes_fees(fees)?;
            farmer_position.add_yes_staked(staked_amount)?;
        } else {
            // no mint
            market_vault.add_total_no_staked(staked_amount)?;
            market_vault.add_total_no_fees(fees)?;
            farmer_position.add_no_staked(staked_amount)?;
        }

        Ok(())
    }
}
