use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::{FarmerPosition, Market},
    utils::{Account, MintInterface, constants, math},
};

pub struct StakeOutcomeToken<'a> {
    accounts: StakeOutcomeTokenAccounts<'a>,
    data: StakeOutcomeTokenData<'a>,
}

struct StakeOutcomeTokenAccounts<'a> {
    farmer: &'a AccountView,
    market: &'a AccountView,
    farmer_position: &'a AccountView,
    outcome_mint: &'a AccountView,
    farmer_ata: &'a AccountView,
    market_outcome_vault: &'a AccountView,
    farmer_position_bump: u8,
}

struct StakeOutcomeTokenData<'a> {
    amount: u64,
    market_id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for StakeOutcomeTokenData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[..8].try_into().unwrap());
        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let market_id = &data[8..];

        Ok(Self { amount, market_id })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for StakeOutcomeTokenAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            farmer,
            market,
            farmer_position,
            outcome_mint,
            farmer_ata,
            market_outcome_vault,
            _token_program,
            _system_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(farmer)?;

        let (farmer_position_address, farmer_position_bump) = Address::find_program_address(
            &[
                constants::FARMER_POSITION_SEED,
                market.address().as_ref(),
                farmer.address().as_ref(),
            ],
            &crate::ID,
        );
        if &farmer_position_address != farmer_position.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (market_outcome_vault_address, _) = Address::find_program_address(
            &[
                constants::MARKET_SEED,
                market.address().as_ref(),
                outcome_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        if &market_outcome_vault_address != market_outcome_vault.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            farmer,
            market,
            farmer_position,
            outcome_mint,
            farmer_ata,
            market_outcome_vault,
            farmer_position_bump,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for StakeOutcomeToken<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: accounts.try_into()?,
            data: data.try_into()?,
        })
    }
}

impl<'a> StakeOutcomeToken<'a> {
    pub const DISCRIMINATOR: &'a u8 = &7;

    pub fn process(&self) -> ProgramResult {
        // check market and its data, and update
        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = Market::load_mut(&mut market_data)?;

        let market_address = Address::derive_address(
            &[constants::MARKET_SEED, self.data.market_id],
            Some(market.bump),
            &crate::ID,
        );
        if &market_address != self.accounts.market.address() {
            return Err(ProgramError::InvalidAccountData);
        }
        if !market.is_open() {
            return Err(ProgramError::InvalidAccountData);
        }

        // checkear manualmente si existe, sino crear. no usando init_if_needed, para ahorrar el find_program_address y pasarle bump si puede a derive_address
        let bump_binding = &[self.accounts.farmer_position_bump];
        let seeds = &[
            Seed::from(constants::FARMER_POSITION_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(self.accounts.farmer.address().as_ref()),
            Seed::from(bump_binding),
        ];
        Account::init_if_needed::<FarmerPosition>(
            self.accounts.farmer_position,
            self.accounts.farmer,
            seeds,
        )?;

        let mut farmer_position_data = self.accounts.farmer_position.try_borrow_mut()?;
        let farmer_position = FarmerPosition::load_mut(&mut farmer_position_data)?;

        // check farmer position
        if !farmer_position.is_initialized {
            farmer_position.set_inner(self.accounts.farmer_position_bump);
        }

        let fees = math::fee_calculation(self.data.amount, market.fee_bps())?;
        let amount_sub_fees = self
            .data
            .amount
            .checked_sub(fees)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // update market and farmer position state
        if &market.outcome_yes_mint() == self.accounts.outcome_mint.address() {
            market.add_yes_fees(fees)?;
            market.add_yes_staked(amount_sub_fees)?;
            farmer_position.add_yes_staked(amount_sub_fees)?;
        } else if &market.outcome_no_mint() == self.accounts.outcome_mint.address() {
            market.add_no_fees(fees)?;
            market.add_no_staked(amount_sub_fees)?;
            farmer_position.add_no_staked(amount_sub_fees)?;
        } else {
            return Err(ProgramError::InvalidAccountData);
        };

        // transfer from farmer to market vault
        MintInterface::transfer(
            self.accounts.farmer_ata,
            self.accounts.market_outcome_vault,
            self.accounts.farmer,
            self.data.amount,
        )
    }
}
