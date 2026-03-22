use pinocchio::{AccountView, Address, ProgramResult, cpi::Seed, error::ProgramError};

use crate::{
    states::{FarmerPosition, Market},
    utils::{Account, MintInterface, constants},
};

pub struct ClaimRewards<'a> {
    accounts: ClaimRewardsAccounts<'a>,
    data: ClaimRewardsData<'a>,
}

struct ClaimRewardsAccounts<'a> {
    farmer: &'a AccountView,
    market: &'a AccountView,
    farmer_position: &'a AccountView,
    reward_mint: &'a AccountView,
    outcome_mint: &'a AccountView,
    market_reward_vault: &'a AccountView,
    market_outcome_vault: &'a AccountView,
    farmer_reward_ata: &'a AccountView,
    farmer_outcome_ata: &'a AccountView,
}

struct ClaimRewardsData<'a> {
    market_id: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for ClaimRewardsData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < constants::MIN_ID_LENGTH {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { market_id: data })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimRewardsAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [
            farmer,
            market,
            farmer_position,
            reward_mint,
            outcome_mint,
            market_reward_vault,
            market_outcome_vault,
            farmer_reward_ata,
            farmer_outcome_ata,
            _token_program,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Account::signer_check(farmer)?;

        let (market_reward_vault_address, _) = Address::find_program_address(
            &[
                constants::MARKET_SEED,
                market.address().as_ref(),
                reward_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        if &market_reward_vault_address != market_reward_vault.address() {
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
            reward_mint,
            outcome_mint,
            market_reward_vault,
            market_outcome_vault,
            farmer_reward_ata,
            farmer_outcome_ata,
        })
    }
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for ClaimRewards<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        Ok(Self {
            accounts: ClaimRewardsAccounts::try_from(accounts)?,
            data: ClaimRewardsData::try_from(data)?,
        })
    }
}

impl<'a> ClaimRewards<'a> {
    pub const DISCRIMINATOR: &'a u8 = &9;

    pub fn process(&self) -> ProgramResult {
        let (staked_amount, reward_amount, market_bump) = {
            // check market and its data
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
            if !market.is_settled() {
                return Err(ProgramError::InvalidAccountData);
            }
            if &market.incentive_mint() != self.accounts.reward_mint.address() {
                return Err(ProgramError::InvalidAccountData);
            }

            // check farmer position and its data (mint, amount)
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
            if !farmer_position.is_initialized {
                return Err(ProgramError::UninitializedAccount);
            }

            // check mint, mint winner, staked amount
            let (staked_amount, reward_amount) = if market.is_resolved_yes()
                && &market.outcome_yes_mint() == self.accounts.outcome_mint.address()
            {
                (
                    farmer_position.yes_staked(),
                    market.calculate_reward(
                        market.total_yes_staked(),
                        farmer_position.yes_staked(),
                    )?,
                )
            } else if market.is_resolved_no()
                && &market.outcome_no_mint() == self.accounts.outcome_mint.address()
            {
                (
                    farmer_position.no_staked(),
                    market
                        .calculate_reward(market.total_no_staked(), farmer_position.no_staked())?,
                )
            } else {
                return Err(ProgramError::InvalidAccountData);
            };

            (staked_amount, reward_amount, market.bump)
        };

        // transfer rewards
        let bump_binding = &[market_bump];
        let seeds = &[
            Seed::from(constants::MARKET_SEED),
            Seed::from(self.data.market_id),
            Seed::from(bump_binding),
        ];

        if reward_amount > 0 {
            MintInterface::transfer_signed(
                self.accounts.market_reward_vault,
                self.accounts.farmer_reward_ata,
                self.accounts.market,
                reward_amount,
                seeds,
            )?;
        }

        // transfer back staked outcome tokens
        if staked_amount > 0 {
            MintInterface::transfer_signed(
                self.accounts.market_outcome_vault,
                self.accounts.farmer_outcome_ata,
                self.accounts.market,
                staked_amount,
                seeds,
            )?;
        }

        // close farmer position account
        Account::close(self.accounts.farmer_position, self.accounts.farmer)
    }
}
