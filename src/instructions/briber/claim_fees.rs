use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::Market,
    utils::{Account, MintInterface, constants},
};

pub struct ClaimFees<'a> {
    accounts: ClaimFeesAccounts<'a>,
    data: ClaimFeesData<'a>,
}

struct ClaimFeesAccounts<'a> {
    briber: &'a AccountView,
    market: &'a AccountView,
    outcome_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_outcome_vault: &'a AccountView,
}

struct ClaimFeesData<'a> {
    id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for ClaimFeesData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        };

        Ok(Self { id: data })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimFeesAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            briber,
            market,       // TODO
            outcome_mint, // TODO
            briber_ata,
            market_outcome_vault,
            _token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(briber)?;

        Ok(Self {
            briber,
            market,
            outcome_mint,
            briber_ata,
            market_outcome_vault,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for ClaimFees<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: accounts.try_into()?,
            data: data.try_into()?,
        })
    }
}

impl<'a> ClaimFees<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(&self) -> ProgramResult {
        // check market and data
        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = Market::load_mut(&mut market_data)?;

        let market_address = Address::derive_address(
            &[constants::MARKET_SEED, self.data.id],
            Some(market.bump),
            &crate::ID,
        );
        if &market_address != self.accounts.market.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        if &market.briber() != self.accounts.briber.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        let (amount, outcome_mint) = if market.is_resolved_yes() {
            (market.available_yes_fees(), market.outcome_yes_mint())
        } else if market.is_resolved_no() {
            (market.available_no_fees(), market.outcome_no_mint())
        } else {
            return Err(ProgramError::InvalidAccountData);
        };

        if amount == 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        if &outcome_mint != self.accounts.outcome_mint.address() {
            return Err(ProgramError::InvalidAccountData);
        }

        // set available fees to 0
        market.clean_available_fees();

        // transfer amount if > 0
        let bump_binding = &[market.bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(bump_binding),
        ];

        MintInterface::transfer_signed(
            self.accounts.market_outcome_vault,
            self.accounts.briber_ata,
            self.accounts.market,
            amount,
            seeds,
        )
    }
}
