use pinocchio::{cpi::Seed, error::ProgramError, AccountView, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address,
    states::MarketVault,
    utils::{Account, MintInterface},
};

pub struct ClaimFees<'a> {
    accounts: ClaimFeesAccounts<'a>,
}

pub struct ClaimFeesAccounts<'a> {
    briber: &'a AccountView,
    market: &'a AccountView,
    outcome_yes_mint: &'a AccountView,
    outcome_no_mint: &'a AccountView,
    briber_outcome_yes: &'a AccountView,
    briber_outcome_no: &'a AccountView,
    outcome_yes_vault: &'a AccountView,
    outcome_no_vault: &'a AccountView,
    // token_program: &'a AccountView,
    // system_program: &'a AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimFeesAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [briber, market, outcome_yes_mint, outcome_no_mint, briber_outcome_yes, briber_outcome_no, outcome_yes_vault, outcome_no_vault, _token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(briber)?;

        Ok(Self {
            briber,
            market,
            outcome_yes_mint,
            outcome_no_mint,
            briber_outcome_yes,
            briber_outcome_no,
            outcome_yes_vault,
            outcome_no_vault,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for ClaimFees<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = ClaimFeesAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> ClaimFees<'a> {
    pub const DISCRIMINATOR: &'a u8 = &9;

    pub fn process(&mut self) -> ProgramResult {
        let (
            is_resolved_yes,
            is_resolved_no,
            total_yes_fees,
            total_no_fees,
            market_id,
            market_bump,
        ) = {
            let mut market_data = self.accounts.market.try_borrow_mut()?;
            let market = MarketVault::load_mut(&mut market_data)?;

            if !market.is_settled() {
                return Err(ReflexError::MarketWasNotSettled.into());
            }
            if market.fees_claimed {
                return Err(ReflexError::FeesAlreadyClaimed.into());
            }
            require_eq_address!(&market.briber, self.accounts.briber.address());

            market.fees_claimed = true;

            (
                market.is_resolved_yes(),
                market.is_resolved_no(),
                market.total_yes_fees(),
                market.total_no_fees(),
                market.id(),
                market.bump,
            )
        };

        let market_id_binding = market_id.to_le_bytes();
        let bump_binding = [market_bump];
        let seeds = [
            Seed::from(constants::MARKET_VAULT_SEED),
            Seed::from(&market_id_binding),
            Seed::from(&bump_binding),
        ];

        if is_resolved_yes && total_yes_fees > 0 {
            MintInterface::transfer_signed(
                self.accounts.outcome_yes_vault,
                self.accounts.briber_outcome_yes,
                self.accounts.market,
                total_yes_fees,
                &seeds,
            )?;
        } else if is_resolved_no && total_no_fees > 0 {
            MintInterface::transfer_signed(
                self.accounts.outcome_no_vault,
                self.accounts.briber_outcome_no,
                self.accounts.market,
                total_no_fees,
                &seeds,
            )?;
        }

        Ok(())
    }
}
