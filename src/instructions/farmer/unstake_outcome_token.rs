use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::{FarmerPosition, Market},
    utils::{Account, MintInterface, constants},
};

pub struct UnstakeOutcomeToken<'a> {
    accounts: UnstakeOutcomeTokenAccounts<'a>,
    data: UnstakeOutcomeTokenData<'a>,
}

struct UnstakeOutcomeTokenAccounts<'a> {
    farmer: &'a AccountView,
    market: &'a AccountView,
    farmer_position: &'a AccountView,
    outcome_mint: &'a AccountView,
    farmer_ata: &'a AccountView,
    market_outcome_vault: &'a AccountView,
}

struct UnstakeOutcomeTokenData<'a> {
    amount: u64,
    market_id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for UnstakeOutcomeTokenData<'a> {
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

impl<'a> TryFrom<&'a [AccountView]> for UnstakeOutcomeTokenAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            farmer,          // TODO
            market,          // TODO
            farmer_position, // TODO
            outcome_mint,    // TODO
            farmer_ata,
            market_outcome_vault, // TODO
            _token_program,
            // _system_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(farmer)?;

        Ok(Self {
            farmer,
            market,
            farmer_position,
            outcome_mint,
            farmer_ata,
            market_outcome_vault,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for UnstakeOutcomeToken<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: UnstakeOutcomeTokenAccounts::try_from(accounts)?,
            data: UnstakeOutcomeTokenData::try_from(data)?,
        })
    }
}

impl<'a> UnstakeOutcomeToken<'a> {
    pub const DISCRIMINATOR: &'a u8 = &8;

    pub fn process(&self) -> ProgramResult {
        // check market and its data ( mint)
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

        // check farmer position and its data (mint)
        let mut farmer_position_data = self.accounts.farmer_position.try_borrow_mut()?;
        let farmer_position = FarmerPosition::load_mut(&mut farmer_position_data)?;

        let farmer_position_address = Address::derive_address(
            &[
                constants::FARMER_POSITION_SEED,
                self.accounts.market.address().as_ref(),
                self.accounts.farmer.address().as_ref(),
            ],
            Some(farmer_position.bump),
            &crate::ID,
        );
        if &farmer_position_address != self.accounts.farmer_position.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        // update accounts
        if &market.outcome_yes_mint() == self.accounts.outcome_mint.address() {
            market.sub_yes_staked(self.data.amount)?;
            farmer_position.sub_yes_staked(self.data.amount)?;
        } else if &market.outcome_no_mint() == self.accounts.outcome_mint.address() {
            market.sub_no_staked(self.data.amount)?;
            farmer_position.sub_no_staked(self.data.amount)?;
        } else {
            return Err(ProgramError::InvalidAccountData);
        }

        // if stakes = 0, close position
        if farmer_position.yes_staked() == 0 && farmer_position.no_staked() == 0 {
            drop(farmer_position_data);
            Account::close(self.accounts.farmer_position, self.accounts.farmer)?;
        }

        // transfer back to farmer ata
        let bump_binding = &[market.bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.data.market_id),
            Seed::from(bump_binding),
        ];

        drop(market_data);

        MintInterface::transfer_signed(
            self.accounts.market_outcome_vault,
            self.accounts.farmer_ata,
            self.accounts.market,
            self.data.amount,
            seeds,
        )
    }
}
