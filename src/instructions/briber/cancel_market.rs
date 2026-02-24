use pinocchio::{cpi::Seed, error::ProgramError, AccountView, Address, ProgramResult};

use crate::{
    constants,
    errors::ReflexError,
    require_eq_address,
    states::MarketVault,
    utils::{Account, MintInterface},
};

pub struct CancelMarket<'a> {
    accounts: CancelMarketAccounts<'a>,
    //  data: ,
}

pub struct CancelMarketAccounts<'a> {
    briber: &'a AccountView,
    market: &'a AccountView,
    briber_ata: &'a AccountView,
    market_incentive_vault: &'a AccountView,
    market_vault_bump: u8,
    // token_program: &'a AccountView,
    // system_program: &'a AccountView,
}

// pub struct CancelMarketData {}

impl<'a> TryFrom<&'a [AccountView]> for CancelMarketAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [briber, market, briber_ata, market_incentive_vault, _token_program, _system_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        Account::signer_check(briber)?;

        let (market_vault_address, market_vault_bump) = Address::find_program_address(
            &[constants::TREASURY_SEED, &market.address().as_ref()],
            &crate::ID,
        );

        require_eq_address!(&market_vault_address, market_incentive_vault.address());

        Ok(Self {
            briber,
            market,
            briber_ata,
            market_incentive_vault,
            market_vault_bump,
        })
    }
}

impl<'a> TryFrom<&'a [AccountView]> for CancelMarket<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = CancelMarketAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> CancelMarket<'a> {
    pub const DISCRIMINATOR: &'a u8 = &5;

    pub fn process(&mut self) -> ProgramResult {
        {
            let market_data = self.accounts.market.try_borrow()?;
            let market = MarketVault::load(&market_data)?;

            // market checks
            require_eq_address!(self.accounts.briber.address(), &market.briber);

            pinocchio_log::log!("market total yes fees: {}", market.total_yes_fees());
            pinocchio_log::log!("market total no fees: {}", market.total_no_fees());

            if market.total_yes_fees() != 0 || market.total_no_fees() != 0 {
                return Err(ReflexError::MarketCanNotBeCancelled.into());
            }

            // transfer incentives to briber
            let bump_binding = self.accounts.market_vault_bump.to_le_bytes();
            let seeds = [
                Seed::from(constants::TREASURY_SEED),
                Seed::from(self.accounts.market.address().as_ref()),
                Seed::from(&bump_binding),
            ];

            MintInterface::transfer_signed(
                self.accounts.market_incentive_vault,
                self.accounts.briber_ata,
                self.accounts.market,
                market.total_incentives(),
                &seeds,
            )?;
        }

        // close market accounts
        Account::close(self.accounts.market, self.accounts.briber)
    }
}
