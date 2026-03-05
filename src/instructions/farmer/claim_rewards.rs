use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address,
    states::{FarmerPosition, MarketVault},
    utils::{Account, MintInterface, TokenAcocuntInterface},
};

pub struct ClaimRewards<'a> {
    accounts: ClaimRewardsAccounts<'a>,
}

pub struct ClaimRewardsAccounts<'a> {
    farmer: &'a AccountView,
    market: &'a AccountView,
    farmer_position: &'a AccountView,
    incentive_mint: &'a AccountView,
    outcome_yes_mint: &'a AccountView,
    outcome_no_mint: &'a AccountView,
    farmer_incentive_ata: &'a AccountView,
    farmer_outcome_yes_ata: &'a AccountView,
    farmer_outcome_no_ata: &'a AccountView,
    market_vault: &'a AccountView,
    outcome_yes_vault: &'a AccountView,
    outcome_no_vault: &'a AccountView,
    // associated_token_program: &'a AccountView,
    // token_program: &'a AccountView,
    // system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimRewardsAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [farmer, market, farmer_position, incentive_mint, outcome_yes_mint, outcome_no_mint, farmer_incentive_ata, farmer_outcome_yes_ata, farmer_outcome_no_ata, market_vault, outcome_yes_vault, outcome_no_vault, _associated_token_program, token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(farmer)?;

        let (farmer_position_address, _) = Address::find_program_address(
            &[
                constants::FARMER_POSITION_SEED,
                market.address().as_ref(),
                farmer.address().as_ref(),
            ],
            &crate::ID,
        );
        require_eq_address!(&farmer_position_address, farmer_position.address());

        TokenAcocuntInterface::ata_check(
            farmer_incentive_ata,
            farmer,
            incentive_mint,
            token_program,
        )?;

        let (market_vault_address, _) = Address::find_program_address(
            &[constants::TREASURY_SEED, market.address().as_ref()],
            &crate::ID,
        );
        require_eq_address!(&market_vault_address, market_vault.address());

        let (outcome_yes_vault_address, _) = Address::find_program_address(
            &[
                constants::TREASURY_SEED,
                market.address().as_ref(),
                outcome_yes_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        require_eq_address!(&outcome_yes_vault_address, outcome_yes_vault.address());

        let (outcome_no_vault_address, _) = Address::find_program_address(
            &[
                constants::TREASURY_SEED,
                market.address().as_ref(),
                outcome_no_mint.address().as_ref(),
            ],
            &crate::ID,
        );
        require_eq_address!(&outcome_no_vault_address, outcome_no_vault.address());

        Ok(Self {
            farmer,
            market,
            farmer_position,
            incentive_mint,
            outcome_yes_mint,
            outcome_no_mint,
            farmer_incentive_ata,
            farmer_outcome_yes_ata,
            farmer_outcome_no_ata,
            market_vault,
            outcome_yes_vault,
            outcome_no_vault,
            // associated_token_program,
            // token_program,
            // system_program,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimRewards<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = ClaimRewardsAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> ClaimRewards<'a> {
    pub const DISCRIMINATOR: &'a u8 = &8;

    pub fn process(&mut self) -> ProgramResult {
        let (
            yes_staked,
            no_staked,
            is_resolved_yes,
            is_resolved_no,
            total_yes_staked,
            total_no_staked,
            total_incentives,
            market_id,
            market_bump,
        ) = {
            let farmer_position_data = self.accounts.farmer_position.try_borrow()?;
            let farmer_position = FarmerPosition::load(&farmer_position_data)?;
            let market_data = self.accounts.market.try_borrow()?;
            let market = MarketVault::load(&market_data)?;

            if !market.is_settled() {
                return Err(ReflexError::MarketWasNotSettled.into());
            }
            require_eq_address!(
                &market.incentive_mint,
                self.accounts.incentive_mint.address()
            );
            require_eq_address!(
                &market.outcome_yes_mint,
                self.accounts.outcome_yes_mint.address()
            );
            require_eq_address!(
                &market.outcome_no_mint,
                self.accounts.outcome_no_mint.address()
            );

            (
                farmer_position.yes_staked(),
                farmer_position.no_staked(),
                market.is_resolved_yes(),
                market.is_resolved_no(),
                market.total_yes_staked(),
                market.total_no_staked(),
                market.total_incentives(),
                market.id(),
                market.bump,
            )
        };

        let market_id_binding = market_id.to_le_bytes();
        let market_bump_binding = [market_bump];
        let market_seeds = [
            Seed::from(constants::MARKET_VAULT_SEED),
            Seed::from(&market_id_binding),
            Seed::from(&market_bump_binding),
        ];

        // and transfer back outcome tokens
        if yes_staked > 0 {
            MintInterface::transfer_signed(
                self.accounts.outcome_yes_vault,
                self.accounts.farmer_outcome_yes_ata,
                self.accounts.market,
                yes_staked,
                &market_seeds,
            )?;
        }
        if no_staked > 0 {
            MintInterface::transfer_signed(
                self.accounts.outcome_no_vault,
                self.accounts.farmer_outcome_no_ata,
                self.accounts.market,
                no_staked,
                &market_seeds,
            )?;
        }

        // calculate rewards based on market resolution and staked amounts
        let (staked_amount, total_staked) = if is_resolved_yes && yes_staked > 0 {
            (yes_staked, total_yes_staked)
        } else if is_resolved_no && no_staked > 0 {
            (no_staked, total_no_staked)
        } else {
            (0, 0)
        };

        let rewards = if staked_amount > 0 {
            (staked_amount as u128)
                .checked_mul(total_incentives as u128)
                .and_then(|v| v.checked_div(total_staked as u128))
                .and_then(|v| v.try_into().ok())
                .ok_or(ProgramError::ArithmeticOverflow)?
        } else {
            0
        };

        // transfer incentives to farmer
        if rewards > 0 {
            MintInterface::transfer_signed(
                self.accounts.market_vault,
                self.accounts.farmer_incentive_ata,
                self.accounts.market,
                rewards,
                &market_seeds,
            )?;
        }

        // close farmer_position account
        Account::close(self.accounts.farmer_position, self.accounts.farmer)
    }
}
