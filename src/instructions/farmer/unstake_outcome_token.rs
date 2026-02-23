use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address, require_eq_len,
    states::{FarmerPosition, MarketVault},
    utils::{Account, MintInterface},
};

pub struct UnstakeOutcomeToken<'a> {
    accounts: UnstakeOutcomeTokenAccounts<'a>,
    data: UnstakeOutcomeTokenData,
}

pub struct UnstakeOutcomeTokenAccounts<'a> {
    farmer: &'a AccountView,
    market: &'a AccountView,
    farmer_position: &'a AccountView,
    outcome_mint: &'a AccountView,
    farmer_outcome_ata: &'a AccountView,
    market_outcome_vault: &'a AccountView,
    bump_market_outcome_vault: u8,
}
pub struct UnstakeOutcomeTokenData {
    amount: u64,
}

impl<'a> TryFrom<&'a [AccountView]> for UnstakeOutcomeTokenAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [farmer, market, farmer_position, outcome_mint, farmer_outcome_ata, market_outcome_vault, _token_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(farmer)?;
        Account::program_account_check(market)?;
        // TokenAcocuntInterface::token_account_check(market_outcome_vault, market, outcome_mint)?; // CHECK: creo que esta mal

        let (market_outcome_vault_address, bump_market_outcome_vault) =
            Address::find_program_address(
                &[
                    constants::TREASURY_SEED,
                    market.address().as_ref(),
                    outcome_mint.address().as_ref(),
                ],
                &crate::ID,
            );

        require_eq_address!(
            &market_outcome_vault_address,
            market_outcome_vault.address()
        );

        let (farmer_position_address, _) = Address::find_program_address(
            &[
                constants::FARMER_POSITION_SEED,
                market.address().as_ref(),
                farmer.address().as_ref(),
            ],
            &crate::ID,
        );

        require_eq_address!(&farmer_position_address, farmer_position.address());

        Ok(Self {
            farmer,
            market,
            farmer_position,
            outcome_mint,
            farmer_outcome_ata,
            market_outcome_vault,
            bump_market_outcome_vault,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for UnstakeOutcomeTokenData {
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

impl<'a> TryFrom<(&'a [u8], &'a [AccountView])> for UnstakeOutcomeToken<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountView])) -> Result<Self, Self::Error> {
        let accounts = UnstakeOutcomeTokenAccounts::try_from(accounts)?;
        let data = UnstakeOutcomeTokenData::try_from(data)?;

        Ok(Self { accounts, data })
    }
}

impl<'a> UnstakeOutcomeToken<'a> {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(&mut self) -> ProgramResult {
        // update market and farmer position accounts
        {
            let mut market_data = self.accounts.market.try_borrow_mut()?;
            let market = MarketVault::load_mut(&mut market_data)?;
            let mut farmer_position_data = self.accounts.farmer_position.try_borrow_mut()?;
            let farmer_position = FarmerPosition::load_mut(&mut farmer_position_data)?;

            if self.accounts.outcome_mint.address() == &market.outcome_yes_mint {
                market.sub_total_yes_staked(self.data.amount)?;
                farmer_position.sub_yes_staked(self.data.amount)?;
            } else if self.accounts.outcome_mint.address() == &market.outcome_no_mint {
                market.sub_total_no_staked(self.data.amount)?;
                farmer_position.sub_no_staked(self.data.amount)?;
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        // transfer from market outcome vault to farmer ata
        let bump_binding = self.accounts.bump_market_outcome_vault.to_le_bytes();
        let seeds = [
            Seed::from(constants::TREASURY_SEED),
            Seed::from(self.accounts.market.address().as_ref()),
            Seed::from(self.accounts.outcome_mint.address().as_ref()),
            Seed::from(&bump_binding),
        ];

        MintInterface::transfer_signed(
            self.accounts.market_outcome_vault,
            self.accounts.farmer_outcome_ata,
            self.accounts.market,
            self.data.amount,
            &seeds,
        )
    }
}
