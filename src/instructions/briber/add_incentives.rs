use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::MarketVault,
    utils::{fee_calculation, Account, MintInterface},
};

pub struct AddIncentives<'a> {
    accounts: AddIncentivesAccounts<'a>,
    data: AddIncentivesData,
}

pub struct AddIncentivesAccounts<'a> {
    briber: &'a AccountView,
    market: &'a AccountView,
    incentive_mint: &'a AccountView,
    briber_ata: &'a AccountView,
    market_incentive_vault: &'a AccountView,
    incentive_treasury: &'a AccountView,
}

pub struct AddIncentivesData {
    amount: u64,
}

impl<'a> TryFrom<&'a [AccountView]> for AddIncentivesAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [briber, market, incentive_mint, briber_ata, market_incentive_vault, incentive_treasury, _token_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(briber)?;

        let (treasury_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, incentive_mint.address().as_ref()],
            &crate::ID,
        );

        require_eq_address!(&treasury_address, incentive_treasury.address());

        Ok(Self {
            briber,
            market,
            incentive_mint,
            briber_ata,
            market_incentive_vault,
            incentive_treasury,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for AddIncentivesData {
    type Error = ProgramError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        require_eq_len!(bytes.len(), size_of::<Self>());

        let amount = u64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| ProgramError::InvalidAccountData)?,
        );

        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { amount })
    }
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for AddIncentives<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = AddIncentivesAccounts::try_from(accounts)?;
        let data = AddIncentivesData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> AddIncentives<'a> {
    pub const DISCRIMINATOR: &'a u8 = &4;

    pub fn process(&mut self) -> ProgramResult {
        // update market account data
        let mut market_data = self.accounts.market.try_borrow_mut()?;
        let market = MarketVault::load_mut(&mut market_data)?;

        require_eq_address!(
            &market.incentive_mint,
            self.accounts.incentive_mint.address()
        );

        require_eq_address!(&market.briber, self.accounts.briber.address());

        if market.is_settled() {
            return Err(ReflexError::MarketWasSetted.into());
        }

        market.add_incentives(self.data.amount)?;

        // transfer fees to treasury
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.incentive_treasury,
            self.accounts.briber,
            fee_calculation(self.data.amount, market.fee_bps())?,
        )?;

        // transfer incentive mint to market account vault
        MintInterface::transfer(
            self.accounts.briber_ata,
            self.accounts.market_incentive_vault,
            self.accounts.briber,
            self.data.amount,
        )
    }
}
